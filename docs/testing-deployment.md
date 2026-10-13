# Testing the Deployment Pipeline Locally

This document describes how to test the Collective Toolbox deployment process (`./deploy`) locally without touching the live production server, eliminating long build times, manual deployment steps, and risk of downtime.

---

## Running the Development CLI

Use `scripts/run-dev-ctb -- <arguments>` instead of choosing a binary manually
or invoking `cargo run`. For example:

```bash
scripts/run-dev-ctb -- csc source destination
scripts/run-dev-ctb --offline -- compress gzip input.txt
```

For a clean working tree, the runner reads each executable's `--version-json`
output and only reuses a build whose full commit SHA equals `git rev-parse HEAD`.
Old, unrelated, malformed, and unreadable builds are rejected. Tracked changes
or untracked files always force a build-system invocation; a matching commit
alone cannot establish freshness for uncommitted sources.

The fallback is `./build --quick --no-test --release` for the native platform,
so Linux uses the supported static musl build rather than a GNU host build.
`--offline` is forwarded to the build script. Without Git history, the runner
builds rather than trusting an unverifiable existing binary. Build diagnostics
go to stderr; command arguments, stdout, exit status, and the caller's working
directory are preserved.
The runner requires Bash, Git, jq, and coreutils, supplied by the development
container/build-tool setup.

Scripts making multiple calls can resolve once with
`scripts/run-dev-ctb --resolve` and execute the returned absolute path. Deployment
uses the same JSON reader, `ctb_binary_commit`, by sourcing the runner; its
target-specific build and `--no-build` policies remain separate.

Compression fixture scripts always use this runner. The compression smoke test
retains its explicit binary argument and `CTOOLBOX_BIN` override for testing
particular releases and cross-target binaries in CI.

## Architecture Overview

The deployment test suite consists of three core components:

1. **Debian 13 (Trixie) Linux QEMU VM (`scripts/linux-vm`)**:
   An isolated virtual machine matching the production server environment:
   - Systemd user session with lingering (`loginctl enable-linger ctb-server`).
   - Privilege separation with three dedicated accounts:
     - `ctb-deployer`: Uploads release artifacts and binaries (owns `/home/ctb-server/deploy` with group setgid `2750`).
     - `ctb-server`: Executes the `ctoolbox` service via `systemd --user`.
     - `ctb-sudoer`: Runs `setup-permissions-groups` and applies `cap_net_bind_service=+ep` capabilities via `sudo`.
   - SSH access configured with Ed25519 key authentication (`ci-logs/VMs/Linux/id_ed25519`).
   - Isolated loopback port forwarding:
     - SSH: `127.0.0.1:2224` -> Guest `22`
     - Web UI/API: `127.0.0.1:8080` -> Guest `8080`
   - **Snapshot Mode**: Defaults to running with QEMU `-snapshot`, meaning all disk writes are written to an ephemeral overlay. Every test run leaves the base disk untouched.

2. **Decoupled & Fast Deployment Options (`./deploy`)**:
   - `--target <live|vm|DIR>`: Switch between the live production server, the local Linux VM sandbox, or a directory of config files.
   - `--reuse-built`: Inspects pre-existing binaries in `built/$target/ctoolbox` using `--version-json`. If a binary matches `git rev-parse HEAD`, it is reused immediately. Only missing or stale targets are compiled.
   - `--no-build`: Strictly requires that binaries already exist and match the current git commit `HEAD`. Bypasses compilation completely (tests run in ~15–30 seconds).
   - `--mock-artifacts`: Generates lightweight mock tarballs for dependency and source packages, skipping multi-hundred-megabyte git archiving and heavy FastCDC chunking.
   - `--dry-run` / `--no-git-bump`: Prevents automatic version increments in `Cargo.toml`, git commits, and git tags during test runs.
   - `--ssh-key <path>`: Uses explicit SSH private key authentication instead of `sshpass`.
   - `--allow-dirty`: Bypasses the clean working tree check when iterating locally.

3. **Automated End-to-End Test Harness (`./scripts/linux-vm test-deploy`)**:
   Starts the VM in snapshot mode, waits for SSH connectivity, executes `./deploy --target vm`, polls `/api/update-status` and verifies the service health, asserts correct directory permissions and capabilities, and prints remote `journalctl --user -u ctoolbox` error logs if the service fails to start.

---

## 1. Initial VM Provisioning

To create and provision the Debian 13 (Trixie) Linux VM image:

```bash
./scripts/setup-linux-vm-image
```

This script:
1. Downloads the Debian 13 generic cloud image (`debian-13-generic-amd64.qcow2`).
2. Generates an Ed25519 keypair at `ci-logs/VMs/Linux/id_ed25519`.
3. Creates a cloud-init NoCloud seed ISO with the three user accounts, packages (`rsync`, `curl`, `libcap2-bin`, `sudo`, `systemd`), and lingering permissions.
4. Boots QEMU once to apply the cloud-init configuration.
5. Saves a clean baseline snapshot named `base-state` in `ci-logs/VMs/Linux/linux.qcow2`.

If working offline, provide a local image path:
```bash
./scripts/setup-linux-vm-image --image /path/to/debian-13.qcow2
```

---

## 2. Managing the Linux VM

The `./scripts/linux-vm` helper provides lifecycle and inspection commands:

```bash
# Start the VM in background with isolated port forwarding (snapshot mode)
./scripts/linux-vm start --net

# Check VM status and process info
./scripts/linux-vm status

# Connect via SSH as ctb-deployer (default)
./scripts/linux-vm ssh

# Connect as a different user (e.g. ctb-server or ctb-sudoer)
./scripts/linux-vm ssh --user ctb-server
./scripts/linux-vm ssh --user ctb-sudoer

# Run a remote command
./scripts/linux-vm run uptime

# List snapshots
./scripts/linux-vm snapshot list

# Revert to clean base-state snapshot
./scripts/linux-vm snapshot revert base-state

# Capture a screenshot of the VM console
./scripts/linux-vm screenshot /tmp/linux-vm.png

# Stop the VM gracefully
./scripts/linux-vm stop
```

---

## 3. Running Deploy Tests

### Fast Integration Test (Using Prebuilt Binaries)

When binaries are already compiled in `built/linux-x64/ctoolbox` and match the current commit:

```bash
# Direct deploy to VM sandbox
./deploy --target vm --no-build --mock-artifacts

# Or run the complete automated verification harness
./scripts/linux-vm test-deploy --no-build --mock-artifacts
```

### Smart Reusable Build & Deploy

If you want to build only what is missing or stale, and reuse existing current binaries:

```bash
./scripts/linux-vm test-deploy --reuse-built --mock-artifacts
```

If the deploy fails midway due to a server configuration issue, you can resolve the issue and rerun without triggering a 30-minute rebuild from scratch.

### Checking Binary Validity and Git Cleanliness

- If you attempt to use `--no-build` with a stale binary (built from an older commit than `HEAD`), `./deploy` will immediately reject it with an error:
  `Error: Binary for target 'linux-x64' is stale (commit 'abc' != HEAD 'def'). Cannot use --no-build.`
- If your working directory has unstaged or uncommitted changes, `./deploy` exits with an error to ensure reproducible releases. For rapid local testing with uncommitted edits, pass `--allow-dirty`.
