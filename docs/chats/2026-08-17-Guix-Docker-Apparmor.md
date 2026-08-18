Presumably the apparmor profile works in some configurations - out of curiosity, are you able to see why it is not compatible when running in a container?



Ran command: `python3 -c "import urllib.request; data = urllib.request.urlopen('https://codeberg.org/guix/guix/raw/branch/master/etc/apparmor.d/guix-daemon').read().decode(); print(data)" 2>/dev/null || true`

The upstream Guix AppArmor profile is designed for a **bare-metal/systemd host**, where it relies on features and assumptions that break inside containers:

---

### 1. Missing `CAP_SETUID` (Assumes User Namespaces)
Looking at the upstream profile definition:
```apparmor
profile guix-daemon ... {
  userns,
  capability sys_admin,
  capability net_admin,
  capability sys_chroot,
  capability setgid,
  capability chown,
  ...
}
```
- **Why it works on a host**: On a normal Linux host with unprivileged user namespaces enabled, `guix-daemon` can create user namespaces (`CLONE_NEWUSER`) and map UID 0 to builder UIDs via `/proc/self/uid_map` without needing `CAP_SETUID`. The profile intentionally omits `capability setuid` as a security hardening measure.
- **Why it breaks in a container**: Containers (Docker/BuildKit) typically disallow or restrict nested unprivileged user namespaces. When `guix-daemon` falls back to traditional `--build-users-group` privilege dropping via standard `setuid()`, AppArmor denies the syscall (`capability=7 capname="setuid"`).

---

### 2. Disconnected Mount Namespaces & Layered OverlayFS
- Guix's AppArmor profile uses `attach_disconnected.path=/disconnected` and strict path rules (`owner /tmp/**`, `@{guix_storedir}/**`).
- Inside Docker/BuildKit, the root filesystem is a nested mount on overlayfs. When `guix-daemon` pivots/chroots into new mount namespaces inside an existing container mount namespace, paths resolve as disconnected or cross-mount objects. This triggers DAC checks (`capability=1 capname="dac_override"` and `capability=4 capname="fsetid"`).

---

### 3. Sub-Profile Confinement on `guix-helper`
- The profile defines strict child transitions (`cx -> guix-helper` for `guix substitute` and `guix download`).
- The `guix-helper` sub-profile restricts helpers to a minimal set of paths. When `guix substitute` in a container interacts with `/tmp` pipelines, local sockets, or unshared network namespaces, AppArmor blocks operations like file capability modifications (`setfcap`).

---

### Summary
On a bare-metal host, `guix-daemon` runs with full root capabilities to set up unprivileged user namespaces and direct access to `/proc` and `/var/guix`. Inside a container sandbox, the AppArmor profile strips the very capabilities (`CAP_SETUID`, DAC overrides) that `guix-daemon` needs to operate without nested user namespaces.