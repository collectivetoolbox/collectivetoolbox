#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

OUT_DIR="$WORKSPACE_ROOT/vendor/v86_images/guix"
OUT_FLAT_DIR="$OUT_DIR/guix-rootfs-flat"
OUT_FS_JSON="$OUT_DIR/guix-fs.json"

export PATH="/var/guix/profiles/per-user/root/current-profile/bin:/root/.config/guix/current/bin:$PATH"

mkdir -p "$OUT_FLAT_DIR"

if ! command -v guix >/dev/null 2>&1; then
    echo "Error: 'guix' command not found on build host."
    echo "Guix is required to build the source-bootstrapped i686 image."
    exit 1
fi

echo "Building ctb-nopersonality shared object..."
cargo rustc -p ctb-nopersonality --release -- --crate-type cdylib
NOPERSONALITY_SO="$WORKSPACE_ROOT/target/release/libctb_nopersonality.so"

echo "Building Guix i686 system tarball image..."
TARBALL_IMG="$(LD_PRELOAD="$NOPERSONALITY_SO" guix system image --system=i686-linux --image-type=tarball "$SCRIPT_DIR/v86-os.scm")"

echo "Guix image built at: $TARBALL_IMG"
echo "Processing image with v86_packer..."

cargo run -p ctb-build-support --bin refresh-asset-bundle --release -- --pack-v86-tar "$TARBALL_IMG" "$OUT_FLAT_DIR" "$OUT_FS_JSON"

if [ ! -f "$OUT_FS_JSON" ] || [ ! -d "$OUT_FLAT_DIR" ] || [ ! -f "$OUT_DIR/guix_posix_initrd.cpio.gz" ]; then
    echo "Error: Failed to produce Guix 9pfs index ($OUT_FS_JSON), flat chunks ($OUT_FLAT_DIR), or initrd archive." >&2
    exit 1
fi

echo "Successfully generated Guix 9pfs index at $OUT_FS_JSON, custom initrd at $OUT_DIR/guix_posix_initrd.cpio.gz, and chunks in $OUT_FLAT_DIR"

