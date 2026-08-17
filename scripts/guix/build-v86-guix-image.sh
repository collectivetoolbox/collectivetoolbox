#!/usr/bin/env bash
set -euxo pipefail

# Build Guix v86 system image and cross-compiled browser packages.
#
# Modes:
#   --build-dillo-native    Build Dillo natively for i686-linux (smoke test)
#   --cross-dillo           Cross-compile Dillo x86_64→i686 (smoke test)
#   --cross-icecat          Cross-compile GNU Icecat x86_64→i686
#   --prebuild-tarball PATH Build Guix i686 system image tarball, save to PATH
#   (no args)               Full build: system image + Icecat + v86 packing

# Container detection helper
is_container() {
    [ -f /.dockerenv ] || [ -f /run/.containerenv ] || [ -f /etc/dockerenv ] || \
    [ -n "${container:-}" ] || [ -n "${DOCKER_BUILD:-}" ] || \
    grep -qa -E 'docker|containerd|kubepods|lxc|podman|buildkit' /proc/1/cgroup 2>/dev/null || \
    grep -qa -E 'docker|containerd|kubepods|lxc|podman|buildkit' /proc/self/cgroup 2>/dev/null || \
    grep -qa -E 'container=' /proc/1/environ 2>/dev/null || \
    grep -qa -E 'container=' /proc/self/environ 2>/dev/null || \
    [ -d /tmp/scripts/guix ]
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "$script_dir/../.." && pwd)"

mode=""
prebuild_dest=""
keep_failed=""
disable_chroot=""
disable_cross=""
no_retries=""

usage() {
    echo "Usage: $0 [--build-dillo-native|--cross-dillo|--cross-icecat|--prebuild-tarball PATH] [--keep-failed] [--disable-chroot] [--disable-cross] [--no-retries]" >&2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --build-dillo-native)
            mode="build-dillo-native"
            ;;
        --cross-dillo)
            mode="cross-dillo"
            ;;
        --cross-icecat)
            mode="cross-icecat"
            ;;
        --prebuild-tarball)
            mode="prebuild-tarball"
            shift
            if [[ $# -eq 0 ]]; then
                echo "Error: --prebuild-tarball requires an output path argument." >&2
                usage
                exit 1
            fi
            prebuild_dest="$1"
            ;;
        --keep-failed)
            keep_failed="--keep-failed"
            ;;
        --disable-chroot)
            disable_chroot="--disable-chroot"
            ;;
        --disable-cross)
            disable_cross="1"
            ;;
        --no-retries)
            no_retries="1"
            ;;
        -h|--help|help)
            usage
            exit 0
            ;;
        *)
            echo "Error: Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
    shift
done

if [[ -n "$disable_cross" ]]; then
    export DISABLE_CROSS=1
fi

if [[ -z "$mode" ]]; then
    mode="full"
fi

out_dir="$workspace_root/vendor/v86_images/guix"
out_flat_dir="$out_dir/guix-rootfs-flat"
out_fs_json="$out_dir/guix-fs.json"

export PATH="/var/guix/profiles/per-user/root/current-profile/bin:/root/.config/guix/current/bin:$PATH"

# Ensure UTF-8 locale environment variables are set to prevent Guile string encoding errors during nar restoration
export GUIX_LOCPATH="${GUIX_LOCPATH:-/var/guix/profiles/per-user/root/current-profile/lib/locale:/root/.guix-profile/lib/locale}"
export LANG="${LANG:-en_US.UTF-8}"
export LC_ALL="${LC_ALL:-en_US.UTF-8}"
export GUILE_AUTO_COMPILE=0
export GUILE_WARN_DEPRECATED=no
if [ -d /root/.cache/guile ]; then
    rm -r /root/.cache/guile 2>/dev/null || true
fi
if [ -n "${HOME:-}" ] && [ -d "$HOME/.cache/guile" ]; then
    rm -r "$HOME/.cache/guile" 2>/dev/null || true
fi
# Configure default Guix build options: allow long compilations without premature silent timeouts
export GUIX_BUILD_OPTIONS="${GUIX_BUILD_OPTIONS:---max-silent-time=3600 --timeout=86400}"

daemon_pid=""
tmp_build_dir=""

# Start guix-daemon, verifying mount namespace support.
# Sets daemon_pid and tmp_build_dir.
start_guix_daemon() {
    if ! command -v guix >/dev/null 2>&1; then
        echo "Error: 'guix' command not found on build host." >&2
        echo "Guix is required to build the source-bootstrapped i686 image." >&2
        exit 1
    fi

    # Check if a system guix-daemon is already running and responding
    if [ -e /var/guix/daemon-socket/socket ] && guix build --dry-run -e '(string-append)' >/dev/null 2>&1; then
        echo "Using existing active guix-daemon."
        daemon_pid=""
        return 0
    fi

    daemon_extra_args=()
    daemon_env=()

    # Container-specific workarounds (UID 0 perform-download patch, nopersonality shim, permissions)
    if is_container; then
        nopersonality_rs=""
        if [ -f "$workspace_root/src/nopersonality/nopersonality.rs" ]; then
            nopersonality_rs="$workspace_root/src/nopersonality/nopersonality.rs"
        elif [ -f "/tmp/src/nopersonality/nopersonality.rs" ]; then
            nopersonality_rs="/tmp/src/nopersonality/nopersonality.rs"
        elif [ -f "$script_dir/../../src/nopersonality/nopersonality.rs" ]; then
            nopersonality_rs="$script_dir/../../src/nopersonality/nopersonality.rs"
        fi

        if [ -n "$nopersonality_rs" ]; then
            nopersonality_dir="/var/guix/nopersonality"
            nopersonality_so="$nopersonality_dir/libctb_nopersonality.so"
            mkdir -p "$nopersonality_dir"
            if [ ! -f "$nopersonality_so" ] || [ "$nopersonality_rs" -nt "$nopersonality_so" ]; then
                echo "Building nopersonality cdylib shim..."
                if command -v rustc >/dev/null 2>&1; then
                    rustc --edition 2024 --crate-type cdylib -O "$nopersonality_rs" -o "$nopersonality_so"
                elif command -v cargo >/dev/null 2>&1; then
                    cargo build --package ctb-nopersonality --release
                    cp "$workspace_root/target/release/libctb_nopersonality.so" "$nopersonality_so"
                fi
            fi
            if [ -f "$nopersonality_so" ]; then
                chmod 755 "$nopersonality_dir" "$nopersonality_so"
                # Also copy to /usr/lib, /gnu/store, and other standard search paths so any chroot can resolve it
                if [ -d /gnu/store ] && [ -w /gnu/store ]; then
                    cp "$nopersonality_so" /gnu/store/libctb_nopersonality.so 2>/dev/null || true
                    chmod 755 /gnu/store/libctb_nopersonality.so 2>/dev/null || true
                fi
                if [ -d /usr/lib ] && [ -w /usr/lib ]; then
                    cp "$nopersonality_so" /usr/lib/libctb_nopersonality.so 2>/dev/null || true
                    chmod 755 /usr/lib/libctb_nopersonality.so 2>/dev/null || true
                fi
                echo "Using nopersonality shim at: $nopersonality_so"
                daemon_env=(env "LD_PRELOAD=$nopersonality_so")
                daemon_extra_args+=(--chroot-directory="$nopersonality_dir")
            fi
        fi

        find /gnu/store -maxdepth 4 -name "perform-download.scm" -exec chmod u+w {} + -exec sed -i 's/(when (zero? (getuid))/(when #f/g' {} + 2>/dev/null || true
        find /gnu/store -maxdepth 4 -name "perform-download.go" -delete 2>/dev/null || true
        find /gnu/store -maxdepth 5 -name "cargo.scm" -path "*/build-system/*" -exec chmod u+w {} + -exec sed -i 's/(default-guile-json)/(cargo-guile-json)/g' {} + 2>/dev/null || true
        find /gnu/store -maxdepth 5 -name "cargo.go" -path "*/build-system/*" -delete 2>/dev/null || true
        if [ -f "$script_dir/patch-guix-substitute.py" ]; then
            python3 "$script_dir/patch-guix-substitute.py" 2>/dev/null || true
        fi

        if [ -d /homeless-shelter ]; then
            rm -r /homeless-shelter 2>/dev/null || true
        fi
    fi

    if [ -d /homeless-shelter ]; then
        rm -r /homeless-shelter 2>/dev/null || true
    fi
    mkdir -p /var/log/guix/drvs /var/guix 2>/dev/null || true
    chown -R root:guixbuild /var/guix /var/log/guix /gnu/store 2>/dev/null || true
    chmod -R 1777 /var/log/guix 2>/dev/null || true
    chmod 1775 /gnu/store /var/guix 2>/dev/null || true

    tmp_build_dir="$(mktemp -d)"
    chmod 755 "$tmp_build_dir"

    if getent group guixbuild >/dev/null 2>&1; then
        daemon_extra_args+=(--build-users-group=guixbuild)
    fi

    if [ -z "$disable_chroot" ]; then
        if ! unshare -m true 2>/dev/null && ! unshare -r -m true 2>/dev/null; then
            echo "Error: Mount namespaces blocked by container environment." >&2
            echo "Guix sandboxing requires mount namespace support." >&2
            exit 1
        fi
    fi

    # Note to LLM tools: Only pass --disable-chroot when explicitly requested via CLI flag.
    ${daemon_env[@]+"${daemon_env[@]}"} guix-daemon ${disable_chroot:+"$disable_chroot"} "${daemon_extra_args[@]}" --max-silent-time=3600 --timeout=86400 --substitute-urls="https://bordeaux.guix.gnu.org https://ci.guix.gnu.org" >/tmp/guix-daemon.log 2>&1 &
    daemon_pid=$!
    sleep 2
    if ! kill -0 "$daemon_pid" 2>/dev/null; then
        if grep -qi -E "Address already in use" /tmp/guix-daemon.log 2>/dev/null; then
            echo "guix-daemon is already running."
            daemon_pid=""
            return 0
        fi
        echo "Error: guix-daemon failed to start." >&2
        cat /tmp/guix-daemon.log >&2 || true
        exit 1
    fi
    echo "guix-daemon started (pid $daemon_pid)."
}

stop_guix_daemon() {
    if [ -n "$daemon_pid" ]; then
        echo "Stopping guix-daemon (pid $daemon_pid)..."
        kill "$daemon_pid" 2>/dev/null || true
        daemon_pid=""
    fi
    if [ -n "$tmp_build_dir" ]; then
        rm -rf "${tmp_build_dir?}" 2>/dev/null || true
        tmp_build_dir=""
    fi
}

# Helper to run guix build/system commands with up to 3 retries for transient network/substitute failures
guix_run_with_retries() {
    local max_attempts=3
    if [ -n "$no_retries" ]; then
        max_attempts=1
    fi
    local attempt=1
    while [ "$attempt" -le "$max_attempts" ]; do
        if [ -d /homeless-shelter ]; then
            rm -r /homeless-shelter 2>/dev/null || true
        fi
        #mkdir -p /var/log/guix/drvs 2>/dev/null || true
        #chmod 1777 /var/log/guix /var/log/guix/drvs 2>/dev/null || true
        if guix "$@"; then
            return 0
        fi
        if [ "$max_attempts" -gt 1 ]; then
            echo "Warning: guix command failed (attempt $attempt of $max_attempts)." >&2
        fi
        attempt=$((attempt + 1))
        if [ "$attempt" -le "$max_attempts" ]; then
            echo "Retrying guix command in 3 seconds..." >&2
            sleep 3
        fi
    done
    if [ "$max_attempts" -gt 1 ]; then
        echo "Error: guix command failed after $max_attempts attempts." >&2
    else
        echo "Error: guix command failed." >&2
    fi
    if [ -f /tmp/guix-daemon.log ]; then
        echo "=== /tmp/guix-daemon.log (last 100 lines) ===" >&2
        tail -n 100 /tmp/guix-daemon.log >&2 || true
    fi
    return 1
}

case "$mode" in
    build-dillo-native)
        start_guix_daemon
        echo "Building Dillo natively for i686-linux..."
        guix_run_with_retries build $keep_failed --fallback -L "$script_dir" --system=i686-linux \
            -e '((@ (patches) apply-patches) (@ (gnu packages web-browsers) dillo))'
        echo "Native Dillo build complete."
        stop_guix_daemon
        ;;

    cross-dillo)
        start_guix_daemon
        echo "Cross-compiling Dillo from x86_64 for i686-linux-gnu..."
        dillo_output="$(guix_run_with_retries build $keep_failed --fallback -L "$script_dir" \
            --system=x86_64-linux --target=i686-linux-gnu \
            -e '((@ (patches) apply-patches) (@ (gnu packages web-browsers) dillo))')"
        dillo_store_path="$(echo "$dillo_output" | grep -o '/gnu/store/[^[:space:]]*dillo-[0-9.]*' | tail -n 1)"
        echo "Cross-compiled Dillo at: $dillo_store_path"
        stop_guix_daemon
        ;;

    cross-icecat)
        start_guix_daemon
        echo "Cross-compiling GNU Icecat from x86_64 for i686-linux-gnu..."
        icecat_output="$(guix_run_with_retries build $keep_failed --fallback -L "$script_dir" \
            --system=x86_64-linux --target=i686-linux-gnu \
            -e '((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat))')"
        icecat_store_path="$(echo "$icecat_output" | grep -o '/gnu/store/[^[:space:]]*icecat-[0-9.]*' | tail -n 1)"
        echo "Cross-compiled Icecat at: $icecat_store_path"
        stop_guix_daemon
        ;;

    prebuild-tarball)
        start_guix_daemon
        echo "Building Guix i686 system tarball image..."
        tarball_output="$(guix_run_with_retries system image $keep_failed --fallback -L "$script_dir" \
            --system=i686-linux --image-type=tarball "$script_dir/v86-os.scm")"
        tarball_img="$(echo "$tarball_output" | grep -o '/gnu/store/[^[:space:]]*\.tar\.gz' | tail -n 1)"
        echo "Guix image built at: $tarball_img"
        stop_guix_daemon
        mkdir -p "$(dirname "$prebuild_dest")"
        cp "$tarball_img" "$prebuild_dest"
        echo "Prebuilt Guix system image tarball at: $prebuild_dest"
        ;;

    full)
        # Full build: used by lint, refresh-asset-bundle, and asset_packer.rs.
        # Builds the system image (or uses a prebuilt one), cross-compiles
        # Icecat, merges Icecat into the rootfs, and packs into v86 format.

        mkdir -p "$out_flat_dir"

        # Check for a prebuilt system tarball.
        prebuilt_tarball=""
        if [ -n "${PREBUILT_V86_TARBALL:-}" ] && [ -f "$PREBUILT_V86_TARBALL" ]; then
            prebuilt_tarball="$PREBUILT_V86_TARBALL"
        elif [ -f "/var/guix/v86-system-image.tar.gz" ]; then
            prebuilt_tarball="/var/guix/v86-system-image.tar.gz"
        elif [ -f "$out_dir/v86-system-image.tar.gz" ]; then
            prebuilt_tarball="$out_dir/v86-system-image.tar.gz"
        fi

        tarball_img=""
        icecat_store_path=""

        # Start daemon if Guix is available — needed for system image build,
        # Icecat cross-compilation, and store closure queries.
        guix_available=0
        if command -v guix >/dev/null 2>&1; then
            guix_available=1
            start_guix_daemon
        fi

        if [ -n "$prebuilt_tarball" ]; then
            echo "Using prebuilt tarball at: $prebuilt_tarball"
            tarball_img="$prebuilt_tarball"
        elif [ "$guix_available" -eq 1 ]; then
            echo "Building Guix i686 system tarball image..."
            tarball_img="$(guix_run_with_retries system image --fallback -L "$script_dir" \
                --system=i686-linux --image-type=tarball "$script_dir/v86-os.scm")"
        else
            echo "Error: No prebuilt tarball found and 'guix' is not available." >&2
            exit 1
        fi

        if [ "$guix_available" -eq 1 ]; then
            if [ -n "${DISABLE_CROSS:-}" ] && [ "$DISABLE_CROSS" = "1" ]; then
                echo "Skipping GNU Icecat cross-compilation (DISABLE_CROSS set)."
            else
                echo "Cross-compiling GNU Icecat from x86_64 for i686-linux-gnu..."
                icecat_store_path="$(guix_run_with_retries build $keep_failed --fallback -L "$script_dir" \
                    --system=x86_64-linux --target=i686-linux-gnu \
                    -e '((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat))' || true)"
                if [ -n "$icecat_store_path" ]; then
                    echo "Cross-compiled Icecat at: $icecat_store_path"
                fi
            fi
        fi

        echo "Guix image at: $tarball_img"

        tmp_rootfs_dir="$(mktemp -d)"
        trap 'rm -rf "${tmp_rootfs_dir?}" 2>/dev/null || true' EXIT

        echo "Extracting Guix system tarball into staging rootfs..."
        tar -xf "$tarball_img" -C "$tmp_rootfs_dir"

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
            echo "Successfully merged Icecat into Guix rootfs!"
        fi

        # Done with all Guix operations; stop daemon before v86 packing.
        stop_guix_daemon

        echo "Processing staging rootfs image with v86_packer..."

        # Unset nested Cargo build environment variables so sub-cargo
        # invocation builds for host cleanly.
        unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_CFG_TARGET_ARCH CARGO_CFG_TARGET_OS 2>/dev/null || true

        cargo run -p ctb-build-support --bin refresh-asset-bundle --release -- \
            --pack-v86-dir "$tmp_rootfs_dir" "$out_flat_dir" "$out_fs_json"

        if [ ! -f "$out_fs_json" ] || [ ! -d "$out_flat_dir" ] || [ ! -f "$out_dir/guix_posix_initrd.cpio.gz" ]; then
            echo "Error: Failed to produce Guix 9pfs index ($out_fs_json), flat chunks ($out_flat_dir), or initrd archive." >&2
            exit 1
        fi

        echo "Successfully generated Guix 9pfs index at $out_fs_json, custom initrd at $out_dir/guix_posix_initrd.cpio.gz, and chunks in $out_flat_dir"
        ;;
esac
