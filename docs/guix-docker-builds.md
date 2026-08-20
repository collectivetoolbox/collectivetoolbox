# Guix in Docker: Architecture, Quirks, and Container Sandboxing

This document details the technical quirks, kernel namespace interactions, AppArmor confinement behaviors, and Docker layer caching strategies encountered when running GNU Guix and building Guix system images inside containerized environments (such as Docker, BuildKit, and CI runners).

---

## 1. Guix Sandboxing vs Container Isolation

GNU Guix uses the Nix daemon (`guix-daemon`) architecture for build isolation. By default, `guix-daemon` expects:
1. **Root execution**: Started as `root` with a pool of unprivileged build user accounts (`guixbuilder01`..`guixbuilder10` in group `guixbuild`).
2. **Kernel namespaces**: Creating private mount (`CLONE_NEWNS`), user (`CLONE_NEWUSER`), UTS (`CLONE_NEWUTS`), and IPC (`CLONE_NEWIPC`) namespaces for every derivation build.
3. **Chroot sandboxing**: Mounting `/gnu/store` read-only and `/tmp` read-write inside a fresh chroot jail.

Inside container environments (like Docker containers or Kubernetes pods), these assumptions clash with the container engine's own security boundaries (seccomp filters, AppArmor/SELinux profiles, and cgroup nesting).

---

## 2. Kernel Namespace & Process Credential Quirks

### A. User Namespace Sync Failure (`in phase usernsInitSync: unexpected end-of-file`)
- **Symptom**:
  ```text
  while setting up the child process: in phase usernsInitSync: unexpected end-of-file
  guix-daemon: nix/libstore/build.cc:4211: void nix::Worker::childTerminated(pid_t, bool): Assertion `i != children.end()' failed.
  ```
- **Cause**:
  When `guix-daemon` runs without `--build-users-group` or when user namespaces are enabled inside a Docker container without full nested user namespace support, `guix-daemon` forks a child and attempts to synchronize UID/GID maps across the user namespace. The kernel denies the nested `clone(CLONE_NEWUSER)` or `setgroups`/`uid_map` write, causing the child to terminate abruptly before `usernsInitSync` completes.
- **Fix**:
  - Always run `guix-daemon` with `--build-users-group=guixbuild`.
  - When mount/user namespaces are disabled by the container host, invoke `guix-daemon` with `--disable-chroot` or verify container runtime capabilities (`--security=insecure` / `--cap-add=SYS_ADMIN`).

### B. `setuid(999)` / `killProcessesForUser(999)` Denials
- **Symptom**:
  ```text
  error: setting uid: Operation not permitted
  guix build: error: cannot kill processes for uid `999': failed with exit code 1
  ```
- **Cause**:
  Before starting and after cleaning up a build, `guix-daemon` attempts to kill all leftover processes owned by the build user via `kill(-1, sig)` and switches credentials using `setuid(uid)` / `setgid(gid)`. Under container seccomp profiles or AppArmor, cross-UID signaling or setuid transitions are blocked.
- **Fix**:
  The `ctb-nopersonality` LD_PRELOAD library (`libctb_nopersonality.so`) intercepts `personality`, `setuid`, `setgid`, `setgroups`, and related capability-dependent syscalls, returning success when running inside container sandboxes.

---

## 3. Host AppArmor Confinement & Path Resolution

### A. AppArmor Profile Attachment via Binary Path
- **Symptom**:
  ```text
  audit: apparmor="DENIED" operation="chmod" class="file" profile="guix-daemon" name="/tmp/guix-build-.../top/" comm="guix-daemon" fsuid=0 ouid=999
  ```
- **Mechanism**:
  Debian/Ubuntu host kernels maintain active AppArmor profiles that match binary paths matching `/**/bin/guix-daemon` or `profile="guix-daemon"`. Even when Docker runs with `--security=insecure` or standard unconfined container settings, the host kernel can match the executable name and transition the container process into the restricted `guix-daemon` profile.
- **The Guile Wrapper Trap**:
  In modern Guix, `/root/.config/guix/current/bin/guix-daemon` is a Guile wrapper script that executes:
  ```scheme
  (apply execl "/gnu/store/<hash>-guix-daemon-<version>/bin/guix-daemon" "guix-daemon" (cdr (command-line)))
  ```
  Simply symlinking or copying the outer script does not help because the Guile wrapper immediately calls `execl` pointing back to the `/gnu/store/.../bin/guix-daemon` path, which matches the host kernel's AppArmor attachment rule.
- **Resolution**:
  1. Inspect the wrapper to extract the underlying ELF binary:
     ```bash
     inner_elf="$(strings "$real_daemon" | grep -o '/gnu/store/[^"]*bin/guix-daemon' | head -n 1)"
     ```
  2. Copy the inner ELF to an unconfined path (e.g. `/var/guix/bin/ctb-daemon`) and execute it directly.
  3. Preload `libctb_nopersonality.so` with shims for `chmod`, `fchmod`, `fchmodat`, `chown`, `fchown`, `lchown`, and `fchownat`. If AppArmor denies directory permission updates during cleanup, the shim safely returns `0` (success).

---

## 4. `guix substitute` Discovery When Bypassing the Wrapper

- **Symptom**:
  ```text
  substitute: error: executing `/gnu/store/...-guix-daemon-.../bin/guix': No such file or directory
  guix build: error: `/gnu/store/...-guix-daemon-.../bin/guix substitute' died unexpectedly
  ```
- **Cause**:
  When `guix-daemon` is launched via the original Guile wrapper, the wrapper exports `GUIX=/gnu/store/...-guix-command`. When running the raw ELF binary directly as `/var/guix/bin/ctb-daemon`, `GUIX` is unset. `guix-daemon` then falls back to searching for `guix` in its own directory (`...-guix-daemon-.../bin/guix`), which does not exist because `guix-daemon` is a separate package output.
- **Fix**:
  Explicitly pass `GUIX` and state paths in the daemon environment:
  ```bash
  local guix_cmd
  guix_cmd="$(type -p guix)"
  daemon_env+=(
      "GUIX=$guix_cmd"
      "GUIX_STATE_DIRECTORY=/var/guix"
      "GUIX_CONFIGURATION_DIRECTORY=/etc/guix"
      "NIX_STORE_DIR=/gnu/store"
  )
  ```

---

## 5. Read-Only `/gnu/store` Symlink Pitfall in Package Overlays

- **Symptom**:
  ```text
  In procedure open-file: Read-only file system: "/tmp/guix-build-.../overlay/bin/llvm-config"
  ```
- **Cause**:
  When creating custom overlay directories in Scheme build phases (such as providing an `llvm-config` wrapper script for Mesa cross-compilation), helper functions like `(symlink-dir-contents src-bin overlay-bin)` recursively symlink all store binaries into the overlay directory first.
  If the phase subsequently attempts `(call-with-output-file wrapper-script ...)` on a path that is already a symlink pointing into the read-only `/gnu/store`, Guile follows the symlink and attempts to write to the store, triggering `EROFS` (`Read-only file system`).
- **Fix**:
  Always delete the existing symlink before creating generated scripts:
  ```scheme
  (when (file-exists? wrapper-script)
    (delete-file wrapper-script))
  (call-with-output-file wrapper-script
    (lambda (p) ...))
  ```

---

## 6. Docker & BuildKit Layer Caching Strategy

`guix pull` downloads and compiles hundreds of megabytes of Scheme channel modules, taking several minutes. To ensure `guix pull` is cached and never invalidated by daily code or patch changes:

### A. Decouple Layer 0 Inputs
- Do **not** `COPY scripts/guix` before `guix pull`.
- Only copy the minimal setup script (`build-v86-guix-image.sh`) and `src/nopersonality` before Layer 0 (`--pull`).
- Any package patches (`scripts/guix/patches/`) and subsequent build steps (`build-v86-guix-image-step2.sh`) must be copied **after** Layer 0.

### B. Two-Stage Script Architecture
```text
scripts/docker/Dockerfile:
  1. COPY scripts/guix/build-v86-guix-image.sh /tmp/scripts/guix/build-v86-guix-image.sh
     COPY src/nopersonality /tmp/src/nopersonality
  2. RUN --security=insecure ... build-v86-guix-image.sh --pull    <-- Layer 0 (PERMANENTLY CACHED)
  3. COPY scripts/guix /tmp/scripts/guix                         <-- Package definitions & step 2
  4. RUN ... build-v86-guix-image-step2.sh --smoke-test-native   <-- Layer 1
  5. RUN ... build-v86-guix-image-step2.sh --smoke-test-cross    <-- Layer 2
  6. RUN ... build-v86-guix-image-step2.sh --cross-dillo         <-- Layer 2b
  7. RUN ... build-v86-guix-image-step2.sh --cross-icecat        <-- Layer 3
  8. RUN ... build-v86-guix-image-step2.sh --prebuild-tarball    <-- Layer 4
```
- Edits to `build-v86-guix-image-step2.sh` or `scripts/guix/patches/*.scm` will only invalidate Layer 1 onwards, keeping Layer 0 fully cached.

---

## 7. Fast Smoke Testing Methodology

To avoid downloading large web browser packages (like Dillo or Icecat) when diagnosing toolchain or sandboxing issues:
- **`--smoke-test-native`**: Runs `guix build --system=i686-linux hello` (< 1 second, < 80 KB download).
- **`--smoke-test-cross`**: Runs `guix build --target=i686-linux-gnu hello` (< 1 second, validates cross-compiler toolchain).

Placing these smoke tests in Layers 1 and 2 allows rapid verification of the Guix daemon and cross-compilation infrastructure before initiating long package builds.
