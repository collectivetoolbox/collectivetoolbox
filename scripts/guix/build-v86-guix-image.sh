#!/usr/bin/env bash
set -euo pipefail

# To build the Guix tarball run ./scripts/guix/build-v86-guix-image.sh --prebuild-tarball path-to-output.tar.gz

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

PREBUILD_DEST=""
if [ "${1:-}" = "--prebuild-tarball" ] && [ -n "${2:-}" ]; then
    PREBUILD_DEST="$2"
fi

OUT_DIR="$WORKSPACE_ROOT/vendor/v86_images/guix"
OUT_FLAT_DIR="$OUT_DIR/guix-rootfs-flat"
OUT_FS_JSON="$OUT_DIR/guix-fs.json"

export PATH="/var/guix/profiles/per-user/root/current-profile/bin:/root/.config/guix/current/bin:$PATH"

if [ -z "$PREBUILD_DEST" ]; then
    mkdir -p "$OUT_FLAT_DIR"
fi

PREBUILT_TARBALL=""
if [ -z "$PREBUILD_DEST" ]; then
    if [ -n "${PREBUILT_V86_TARBALL:-}" ] && [ -f "$PREBUILT_V86_TARBALL" ]; then
        PREBUILT_TARBALL="$PREBUILT_V86_TARBALL"
    elif [ -f "/var/guix/v86-system-image.tar.gz" ]; then
        PREBUILT_TARBALL="/var/guix/v86-system-image.tar.gz"
    elif [ -f "$OUT_DIR/v86-system-image.tar.gz" ]; then
        PREBUILT_TARBALL="$OUT_DIR/v86-system-image.tar.gz"
    fi
fi

if [ -n "$PREBUILT_TARBALL" ]; then
    echo "Using prebuilt Guix system image tarball at: $PREBUILT_TARBALL"
    TARBALL_IMG="$PREBUILT_TARBALL"
else
    if ! command -v guix >/dev/null 2>&1; then
        echo "Error: 'guix' command not found on build host."
        echo "Guix is required to build the source-bootstrapped i686 image."
        exit 1
    fi

    echo "Building ctb-nopersonality shared object in Rust..."
    TMP_BUILD_DIR="$(mktemp -d)"
    NOPERSONALITY_SO="$TMP_BUILD_DIR/libctb_nopersonality.so"
    if [ -f "$WORKSPACE_ROOT/src/nopersonality/nopersonality.rs" ]; then
        NOPERSONALITY_RS="$WORKSPACE_ROOT/src/nopersonality/nopersonality.rs"
    else
        NOPERSONALITY_RS="$(cd "$SCRIPT_DIR/../../src/nopersonality" 2>/dev/null && pwd)/nopersonality.rs"
    fi

    pkill -f guix-daemon 2>/dev/null || true
    sleep 1

    echo "Starting guix-daemon with nopersonality shim..."
    LD_PRELOAD="$NOPERSONALITY_SO" guix-daemon --disable-chroot --build-users-group=guixbuild >/tmp/guix-daemon.log 2>&1 &
    DAEMON_PID=$!
    sleep 2
    if ! kill -0 "$DAEMON_PID" 2>/dev/null; then
        echo "Error: guix-daemon failed to start with nopersonality shim." >&2
        cat /tmp/guix-daemon.log >&2 || true
        exit 1
    fi

    echo "Building Guix i686 system tarball image..."
    TARBALL_IMG="$(guix system image -L "$SCRIPT_DIR" --system=i686-linux --image-type=tarball "$SCRIPT_DIR/v86-os.scm")"

    kill "$DAEMON_PID" 2>/dev/null || true
    rm -rf "${TMP_BUILD_DIR?}" 2>/dev/null || true
fi

if [ -n "$PREBUILD_DEST" ]; then
    mkdir -p "$(dirname "$PREBUILD_DEST")"
    cp "$TARBALL_IMG" "$PREBUILD_DEST"
    echo "Successfully prebuilt Guix system image tarball at: $PREBUILD_DEST"
    exit 0
fi

echo "Guix image built at: $TARBALL_IMG"
echo "Processing image with v86_packer..."

# Unset nested Cargo build environment variables so sub-cargo invocation builds for host cleanly
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_CFG_TARGET_ARCH CARGO_CFG_TARGET_OS 2>/dev/null || true

cargo run -p ctb-build-support --bin refresh-asset-bundle --release -- --pack-v86-tar "$TARBALL_IMG" "$OUT_FLAT_DIR" "$OUT_FS_JSON"

if [ ! -f "$OUT_FS_JSON" ] || [ ! -d "$OUT_FLAT_DIR" ] || [ ! -f "$OUT_DIR/guix_posix_initrd.cpio.gz" ]; then
    echo "Error: Failed to produce Guix 9pfs index ($OUT_FS_JSON), flat chunks ($OUT_FLAT_DIR), or initrd archive." >&2
    exit 1
fi

echo "Successfully generated Guix 9pfs index at $OUT_FS_JSON, custom initrd at $OUT_DIR/guix_posix_initrd.cpio.gz, and chunks in $OUT_FLAT_DIR"
