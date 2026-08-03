#!/usr/bin/env bash
set -euo pipefail

# To build the Guix tarball run ./scripts/guix/build-v86-guix-image.sh --prebuild-tarball path-to-output.tar.gz
# To cross-compile icecat only run ./scripts/guix/build-v86-guix-image.sh --cross-icecat

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "$script_dir/../.." && pwd)"

fetch_sources_mode=0
build_icecat_only=0
prebuild_dest=""

if [ "${1:-}" = "--fetch-sources" ]; then
    fetch_sources_mode=1
elif [ "${1:-}" = "--cross-icecat" ]; then
    build_icecat_only=1
elif [ "${1:-}" = "--prebuild-tarball" ] && [ -n "${2:-}" ]; then
    prebuild_dest="$2"
fi

out_dir="$workspace_root/vendor/v86_images/guix"
out_flat_dir="$out_dir/guix-rootfs-flat"
out_fs_json="$out_dir/guix-fs.json"

export PATH="/var/guix/profiles/per-user/root/current-profile/bin:/root/.config/guix/current/bin:$PATH"

if [ -z "$prebuild_dest" ] && [ "$build_icecat_only" -eq 0 ]; then
    mkdir -p "$out_flat_dir"
fi

prebuilt_tarball=""
if [ -z "$prebuild_dest" ] && [ "$fetch_sources_mode" -eq 0 ] && [ "$build_icecat_only" -eq 0 ]; then
    if [ -n "${PREBUILT_V86_TARBALL:-}" ] && [ -f "$PREBUILT_V86_TARBALL" ]; then
        prebuilt_tarball="$PREBUILT_V86_TARBALL"
    elif [ -f "/var/guix/v86-system-image.tar.gz" ]; then
        prebuilt_tarball="/var/guix/v86-system-image.tar.gz"
    elif [ -f "$out_dir/v86-system-image.tar.gz" ]; then
        prebuilt_tarball="$out_dir/v86-system-image.tar.gz"
    fi
fi

tarball_img=""

if [ -n "$prebuilt_tarball" ]; then
    echo "Using prebuilt Guix system image tarball at: $prebuilt_tarball"
    tarball_img="$prebuilt_tarball"
else
    if ! command -v guix >/dev/null 2>&1; then
        echo "Error: 'guix' command not found on build host."
        echo "Guix is required to build the source-bootstrapped i686 image."
        exit 1
    fi

    echo "Building ctb-nopersonality shared object in Rust..."
    tmp_build_dir="$(mktemp -d)"
    nopersonality_so="$tmp_build_dir/libctb_nopersonality.so"
    if [ -f "$workspace_root/src/nopersonality/nopersonality.rs" ]; then
        nopersonality_rs="$workspace_root/src/nopersonality/nopersonality.rs"
    else
        nopersonality_rs="$(cd "$script_dir/../../src/nopersonality" 2>/dev/null && pwd)/nopersonality.rs"
    fi
    rustc --edition 2024 --crate-type cdylib -O "$nopersonality_rs" -o "$nopersonality_so"

    chown -R root:guixbuild /var/guix /gnu/store 2>/dev/null || true
    chmod 1775 /gnu/store /var/guix 2>/dev/null || true

    echo "Starting guix-daemon with nopersonality shim..."
    mkdir -p /var/tmp/proot_tmp
    export PROOT_TMP_DIR=/var/tmp/proot_tmp
    if command -v proot >/dev/null 2>&1; then
        python3 -c "import ctypes, os; libc = ctypes.CDLL(None); res = libc.ptrace(0, 0, 0, 0); print('PTRACE RESULT:', res); assert res == 0, 'ptrace blocked by seccomp!'"
        echo "Done 0"
        # PROOT_NO_SECCOMP=1 proot -0 env python3 -c "import ctypes, os; libc = ctypes.CDLL(None); res = libc.ptrace(0, 0, 0, 0); print('PTRACE RESULT:', res); assert res == 0, 'ptrace blocked by seccomp!'" 2>&1
        # echo "Done 1"
        # PROOT_NO_SECCOMP=1 proot -0 env LD_PRELOAD="$nopersonality_so" python3 -c "import ctypes, os; libc = ctypes.CDLL(None); res = libc.ptrace(0, 0, 0, 0); print('PTRACE RESULT:', res); assert res == 0, 'ptrace blocked by seccomp!'" 2>&1
        # echo "Done 2"
        LD_PRELOAD="$nopersonality_so" guix-daemon --build-users-group=guixbuild >/tmp/guix-daemon.log 2>&1 &
        # PROOT_NO_SECCOMP=1 proot -0 env LD_PRELOAD="$nopersonality_so" guix-daemon --build-users-group=guixbuild >/tmp/guix-daemon.log 2>&1 &
    else
        LD_PRELOAD="$nopersonality_so" guix-daemon --build-users-group=guixbuild >/tmp/guix-daemon.log 2>&1 &
    fi
    daemon_pid=$!
    sleep 2
    if ! kill -0 "$daemon_pid" 2>/dev/null; then
        echo "Error: guix-daemon failed to start with nopersonality shim." >&2
        cat /tmp/guix-daemon.log >&2 || true
        exit 1
    fi

    if [ "$fetch_sources_mode" -eq 1 ]; then
        echo "Pre-fetching all transitive sources for Guix system image..."
        guix build --sources=transitive -L "$script_dir" --system=i686-linux -e '((@ (gnu system) operating-system-packages) (load "'"$script_dir"'/v86-os.scm"))'
        echo "Pre-fetching transitive sources for cross-compiling Icecat..."
        guix build --sources=transitive --system=x86_64-linux --target=i686-linux-gnu icecat || true
        kill "$daemon_pid" 2>/dev/null || true
        rm -rf "${tmp_build_dir?}" 2>/dev/null || true
        echo "Successfully pre-fetched all system sources."
        exit 0
    fi

    if [ "$build_icecat_only" -eq 1 ]; then
        echo "Cross-compiling GNU Icecat from host (x86_64) for i686-linux-gnu..."
        icecat_store_path="$(guix build --system=x86_64-linux --target=i686-linux-gnu icecat)"
        echo "Cross-compiled Icecat at: $icecat_store_path"
        kill "$daemon_pid" 2>/dev/null || true
        rm -rf "${tmp_build_dir?}" 2>/dev/null || true
        exit 0
    fi

    echo "Building Guix i686 system tarball image..."
    tarball_img="$(guix system image -L "$script_dir" --system=i686-linux --image-type=tarball "$script_dir/v86-os.scm")"

    echo "Cross-compiling GNU Icecat from host (x86_64) for i686-linux-gnu..."
    icecat_store_path="$(guix build --system=x86_64-linux --target=i686-linux-gnu icecat)"
    echo "Cross-compiled Icecat at: $icecat_store_path"

    kill "$daemon_pid" 2>/dev/null || true
    rm -rf "${tmp_build_dir?}" 2>/dev/null || true
fi

if [ -n "$prebuild_dest" ]; then
    mkdir -p "$(dirname "$prebuild_dest")"
    cp "$tarball_img" "$prebuild_dest"
    echo "Successfully prebuilt Guix system image tarball at: $prebuild_dest"
    exit 0
fi

echo "Guix image built at: $tarball_img"

tmp_rootfs_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_rootfs_dir?}" 2>/dev/null || true' EXIT

echo "Extracting Guix system tarball image into staging rootfs..."
tar -xf "$tarball_img" -C "$tmp_rootfs_dir"

if command -v guix >/dev/null 2>&1; then
    if [ -z "${icecat_store_path:-}" ]; then
        echo "Cross-compiling Icecat for i686-linux-gnu..."
        icecat_store_path="$(guix build --system=x86_64-linux --target=i686-linux-gnu icecat || true)"
    fi

    if [ -n "${icecat_store_path:-}" ] && [ -d "$icecat_store_path" ]; then
        echo "Merging cross-compiled Icecat closure ($icecat_store_path) into rootfs..."
        icecat_closure="$(guix gc -R "$icecat_store_path")"
        mkdir -p "$tmp_rootfs_dir/gnu/store"
        for store_item in $icecat_closure; do
            if [ -e "$store_item" ]; then
                cp -a "$store_item" "$tmp_rootfs_dir/gnu/store/"
            fi
        done

        sys_profile="$(find "$tmp_rootfs_dir/gnu/store" -maxdepth 1 -name "*-profile" | head -n 1 || true)"
        if [ -n "$sys_profile" ] && [ -d "$sys_profile/bin" ]; then
            ln -sf "$icecat_store_path/bin/icecat" "$sys_profile/bin/icecat"
        fi
        mkdir -p "$tmp_rootfs_dir/usr/local/bin"
        ln -sf "$icecat_store_path/bin/icecat" "$tmp_rootfs_dir/usr/local/bin/icecat"
        echo "Successfully merged Icecat into Guix rootfs profile!"
    fi
fi

echo "Processing staging rootfs image with v86_packer..."

# Unset nested Cargo build environment variables so sub-cargo invocation builds for host cleanly
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_CFG_TARGET_ARCH CARGO_CFG_TARGET_OS 2>/dev/null || true

cargo run -p ctb-build-support --bin refresh-asset-bundle --release -- --pack-v86-dir "$tmp_rootfs_dir" "$out_flat_dir" "$out_fs_json"

if [ ! -f "$out_fs_json" ] || [ ! -d "$out_flat_dir" ] || [ ! -f "$out_dir/guix_posix_initrd.cpio.gz" ]; then
    echo "Error: Failed to produce Guix 9pfs index ($out_fs_json), flat chunks ($out_flat_dir), or initrd archive." >&2
    exit 1
fi

echo "Successfully generated Guix 9pfs index at $out_fs_json, custom initrd at $out_dir/guix_posix_initrd.cpio.gz, and chunks in $out_flat_dir"


