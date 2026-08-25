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

# Note that it will CLEAR your Guile cache. (rm -r .cache/guile)

# To invoke from within Dev Container: `scripts/guix/build-v86-guix-image.sh --cross-icecat --disable-chroot`

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
        --pull)
            mode="pull"
            ;;
        --smoke-test-native)
            mode="smoke-test-native"
            ;;
        --smoke-test-cross|--smoke-test)
            mode="smoke-test-cross"
            ;;
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
# if is_container; then
    if [ -d /root/.cache/guile ]; then
        rm -r /root/.cache/guile 2>/dev/null || true
    fi
    if [ -n "${HOME:-}" ] && [ -d "$HOME/.cache/guile" ]; then
        rm -r "$HOME/.cache/guile" 2>/dev/null || true
    fi
# fi
# Configure default Guix build options: allow long compilations without premature silent timeouts
export GUIX_BUILD_OPTIONS="${GUIX_BUILD_OPTIONS:---max-silent-time=3600 --timeout=86400}"

daemon_pid=""
tmp_build_dir=""
monitor_pid=""
daemon_tail_pid=""

export CTB_NOPERSONALITY_DEBUG=1

# Print comprehensive system and environment diagnostics
print_system_diagnostics() {
    echo "=== [diagnostic] Build Host Diagnostics ($(date -u '+%Y-%m-%d %H:%M:%S UTC')) ==="
    echo "[diagnostic] Kernel: $(uname -a)"
    echo "[diagnostic] User: $(id)"
    echo "[diagnostic] CPUs: $(nproc 2>/dev/null || echo '?')"
    if [ -f /proc/loadavg ]; then
        echo "[diagnostic] Load avg: $(cat /proc/loadavg)"
    fi
    if [ -f /proc/meminfo ]; then
        echo "[diagnostic] Memory Info:"
        grep -E '^(MemTotal|MemFree|MemAvailable|SwapTotal|SwapFree|Cached|Buffers):' /proc/meminfo | sed 's/^/[diagnostic]   /'
    fi
    if [ -f /sys/fs/cgroup/memory.max ]; then
        echo "[diagnostic] cgroup v2 memory: max=$(cat /sys/fs/cgroup/memory.max 2>/dev/null), current=$(cat /sys/fs/cgroup/memory.current 2>/dev/null)"
    elif [ -f /sys/fs/cgroup/memory/memory.limit_in_bytes ]; then
        echo "[diagnostic] cgroup v1 memory: limit=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>/dev/null), usage=$(cat /sys/fs/cgroup/memory/memory.usage_in_bytes 2>/dev/null)"
    fi
    echo "[diagnostic] Limits:"
    ulimit -a | sed 's/^/[diagnostic]   /'
    echo "============================================================"
}

start_resource_monitor() {
    (
        while true; do
            sleep 2
            local mem_info=""
            if [ -f /proc/meminfo ]; then
                local mem_avail mem_free swap_free
                mem_avail="$(grep -i MemAvailable /proc/meminfo | awk '{print $2, $3}' || true)"
                mem_free="$(grep -i MemFree /proc/meminfo | awk '{print $2, $3}' || true)"
                swap_free="$(grep -i SwapFree /proc/meminfo | awk '{print $2, $3}' || true)"
                mem_info="MemAvail: ${mem_avail:-?}, MemFree: ${mem_free:-?}, SwapFree: ${swap_free:-?}"
            fi
            local cgroup_info=""
            if [ -f /sys/fs/cgroup/memory.current ]; then
                local cur max
                cur="$(cat /sys/fs/cgroup/memory.current 2>/dev/null || true)"
                max="$(cat /sys/fs/cgroup/memory.max 2>/dev/null || true)"
                cgroup_info="cgroup_mem: $cur/$max"
            elif [ -f /sys/fs/cgroup/memory/memory.usage_in_bytes ]; then
                local cur max
                cur="$(cat /sys/fs/cgroup/memory/memory.usage_in_bytes 2>/dev/null || true)"
                max="$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>/dev/null || true)"
                cgroup_info="cgroup_mem: $cur/$max"
            fi
            local oom_info=""
            if [ -f /proc/vmstat ]; then
                local oom_cnt
                oom_cnt="$(grep -i oom_kill /proc/vmstat 2>/dev/null | awk '{print $2}' || true)"
                if [ -n "$oom_cnt" ] && [ "$oom_cnt" -gt 0 ]; then
                    oom_info=" [OOM_KILL_COUNT: $oom_cnt]"
                fi
            fi
            local top_proc=""
            if command -v ps >/dev/null 2>&1; then
                top_proc="$(ps -eo comm,rss --sort=-rss 2>/dev/null | sed -n '2p' | awk '{print "top:" $1 "(" $2 "KB)"}' || true)"
            fi
            local guix_procs
            guix_procs="$(pgrep -c -f 'guix' 2>/dev/null || echo 0)"
            echo "[monitor $(date +%T)] $mem_info | $cgroup_info | guix procs: $guix_procs | $top_proc$oom_info" >&2
        done
    ) &
    monitor_pid=$!
}

stop_resource_monitor() {
    if [ -n "$monitor_pid" ]; then
        kill "$monitor_pid" 2>/dev/null || true
        wait "$monitor_pid" 2>/dev/null || true
        monitor_pid=""
    fi
}

start_daemon_tail() {
    touch /tmp/guix-daemon.log
    tail -n 0 -F /tmp/guix-daemon.log 2>/dev/null | while read -r line; do
        echo "[guix-daemon] $line" >&2
    done &
    daemon_tail_pid=$!
}

stop_daemon_tail() {
    if [ -n "$daemon_tail_pid" ]; then
        kill "$daemon_tail_pid" 2>/dev/null || true
        wait "$daemon_tail_pid" 2>/dev/null || true
        daemon_tail_pid=""
    fi
}

on_script_exit() {
    local exit_code=$?
    trap - EXIT ERR INT TERM HUP QUIT ABRT
    stop_resource_monitor
    stop_daemon_tail
    stop_guix_daemon
    if [ "$exit_code" -ne 0 ]; then
        echo "=== [diagnostic] Build script failed/terminated with exit code $exit_code ===" >&2
        if [ -f /proc/meminfo ]; then
            echo "=== [diagnostic] /proc/meminfo at failure ===" >&2
            cat /proc/meminfo 2>/dev/null | head -n 15 >&2 || true
        fi
        if [ -f /proc/vmstat ]; then
            echo "=== [diagnostic] /proc/vmstat oom ===" >&2
            grep -i oom /proc/vmstat 2>/dev/null >&2 || true
        fi
        if [ -f /sys/fs/cgroup/memory.events ]; then
            echo "=== [diagnostic] cgroup v2 memory events ===" >&2
            cat /sys/fs/cgroup/memory.events >&2 || true
        fi
        if [ -f /sys/fs/cgroup/memory/memory.oom_control ]; then
            echo "=== [diagnostic] cgroup v1 oom control ===" >&2
            cat /sys/fs/cgroup/memory/memory.oom_control >&2 || true
        fi
        if command -v dmesg >/dev/null 2>&1; then
            echo "=== [diagnostic] dmesg (last 30 lines) ===" >&2
            dmesg 2>/dev/null | tail -n 30 >&2 || true
        fi
        if [ -f /tmp/guix-daemon.log ]; then
            echo "=== [diagnostic] /tmp/guix-daemon.log (last 100 lines) ===" >&2
            tail -n 100 /tmp/guix-daemon.log >&2 || true
        fi
        if [ -d /var/log/guix/drvs ]; then
            echo "=== [diagnostic] Recent failed/modified derivation build logs ===" >&2
            find /var/log/guix/drvs -type f -name "*.drv*" -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -n 5 | while read -r _ logfile; do
                echo "=== [diagnostic] Derivation Log: $logfile ===" >&2
                if [[ "$logfile" == *.gz ]]; then
                    gzip -dc "$logfile" 2>/dev/null | tail -n 80 >&2 || true
                else
                    tail -n 80 "$logfile" 2>/dev/null >&2 || true
                fi
            done
        fi
        echo "=== [diagnostic] Process snapshot ===" >&2
        ps aux 2>/dev/null | head -n 30 >&2 || true
    fi
    exit "$exit_code"
}
trap on_script_exit EXIT ERR INT TERM HUP QUIT ABRT

print_system_diagnostics
# start_resource_monitor

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
            nopersonality_dir="/usr/lib"
            nopersonality_so="$nopersonality_dir/libctb_nopersonality.so"
            if [ ! -w "$nopersonality_dir" ]; then
                nopersonality_dir="/var/guix/nopersonality"
                nopersonality_so="$nopersonality_dir/libctb_nopersonality.so"
                mkdir -p "$nopersonality_dir"
            fi
            if [ ! -f "$nopersonality_so" ] || [ "$nopersonality_rs" -nt "$nopersonality_so" ]; then
                echo "Building nopersonality cdylib shim..."
                if command -v rustc >/dev/null 2>&1; then
                    rustc --edition 2024 --crate-type cdylib -C panic=abort -O "$nopersonality_rs" -o "$nopersonality_so"
                elif command -v cargo >/dev/null 2>&1; then
                    cargo build --package ctb-nopersonality --release
                    cp "$workspace_root/target/release/libctb_nopersonality.so" "$nopersonality_so"
                fi
            fi
            if [ -f "$nopersonality_so" ]; then
                chmod 755 "$nopersonality_so"
                mkdir -p /var/guix/nopersonality 2>/dev/null || true
                cp "$nopersonality_so" /var/guix/nopersonality/libctb_nopersonality.so 2>/dev/null || true
                if [ -d /gnu/store ] && [ -w /gnu/store ]; then
                    cp "$nopersonality_so" /gnu/store/libctb_nopersonality.so 2>/dev/null || true
                    chmod 755 /gnu/store/libctb_nopersonality.so 2>/dev/null || true
                fi
                if [ -f /gnu/store/libctb_nopersonality.so ]; then
                    daemon_env=(env "LD_PRELOAD=/gnu/store/libctb_nopersonality.so")
                else
                    daemon_env=(env "LD_PRELOAD=$nopersonality_so")
                fi
            fi
        fi



        if [ -d /homeless-shelter ]; then
            rm -r /homeless-shelter 2>/dev/null || true
        fi
    fi

    if is_container; then
        if [ -d /homeless-shelter ]; then
            rm -r /homeless-shelter 2>/dev/null || true
        fi
        mkdir -p /var/log/guix/drvs /var/guix 2>/dev/null || true
        chown -R root:guixbuild /var/guix /var/log/guix /gnu/store 2>/dev/null || true
        chmod -R 1777 /var/log/guix 2>/dev/null || true
        chmod 1775 /gnu/store /var/guix 2>/dev/null || true
    fi

    tmp_build_dir="$(mktemp -d)"
    chmod 755 "$tmp_build_dir"

    if getent group guixbuild >/dev/null 2>&1; then
        daemon_extra_args+=(--build-users-group=guixbuild)
    fi

    if [ -z "$disable_chroot" ]; then
        if ! unshare -m true 2>/dev/null && ! unshare -r -m true 2>/dev/null; then
            echo "Mount namespaces unavailable in this environment. Automatically enabling --disable-chroot." >&2
            disable_chroot="--disable-chroot"
        fi
    fi

    local daemon_bin
    daemon_bin="$(type -p guix-daemon)"
    if is_container; then
        mkdir -p /var/guix/bin
        local real_daemon
        real_daemon="$(readlink -f "$daemon_bin")"
        if file "$real_daemon" 2>/dev/null | grep -qi "script"; then
            local inner_elf
            inner_elf="$(strings "$real_daemon" 2>/dev/null | grep -o '/gnu/store/[^"]*bin/guix-daemon' | head -n 1)"
            if [ -n "$inner_elf" ] && [ -f "$inner_elf" ]; then
                real_daemon="$inner_elf"
            fi
        fi
        cp -f "$real_daemon" /var/guix/bin/ctb-daemon
        chmod 755 /var/guix/bin/ctb-daemon
        daemon_bin="/var/guix/bin/ctb-daemon"
        if command -v aa-exec >/dev/null 2>&1; then
            daemon_env=(aa-exec -p unconfined "${daemon_env[@]}")
        fi
    fi

    local guix_cmd
    guix_cmd="$(type -p guix)"
    daemon_env+=("GUIX=$guix_cmd" "GUIX_STATE_DIRECTORY=/var/guix" "GUIX_CONFIGURATION_DIRECTORY=/etc/guix" "NIX_STORE_DIR=/gnu/store")

    # Note to LLM tools: Only pass --disable-chroot when explicitly requested via CLI flag.
    ${daemon_env[@]+"${daemon_env[@]}"} "$daemon_bin" ${disable_chroot:+"$disable_chroot"} --debug "${daemon_extra_args[@]}" --max-silent-time=3600 --timeout=86400 --substitute-urls="https://bordeaux.guix.gnu.org https://ci.guix.gnu.org" >/tmp/guix-daemon.log 2>&1 &
    daemon_pid=$!
    start_daemon_tail
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
    stop_daemon_tail
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

# Helper to run guix build/system commands (defaults to 1 attempt; set GUIX_MAX_ATTEMPTS to retry)
guix_run_with_retries() {
    local max_attempts="${GUIX_MAX_ATTEMPTS:-1}"
    if [ -n "$no_retries" ]; then
        max_attempts=1
    fi
    local attempt=1
    while [ "$attempt" -le "$max_attempts" ]; do
        if [ -d /homeless-shelter ]; then
            rm -r /homeless-shelter 2>/dev/null || true
        fi
        echo "[$(date +%T)] Starting guix command (attempt $attempt of $max_attempts): guix $*" >&2
        local start_ts
        start_ts="$(date +%s)"
        local exit_st=0
        if guix "$@"; then
            local end_ts
            end_ts="$(date +%s)"
            echo "[$(date +%T)] guix command succeeded in $((end_ts - start_ts))s." >&2
            return 0
        else
            exit_st=$?
        fi
        local end_ts
        end_ts="$(date +%s)"
        echo "[$(date +%T)] guix command failed with exit status $exit_st after $((end_ts - start_ts))s." >&2
        if [ -d /var/log/guix/drvs ]; then
            echo "=== [diagnostic] Recent Derivation Logs after command failure ===" >&2
            find /var/log/guix/drvs -type f -name "*.drv*" -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -n 3 | while read -r _ logfile; do
                echo "--- Derivation Log: $logfile ---" >&2
                if [[ "$logfile" == *.gz ]]; then
                    gzip -dc "$logfile" 2>/dev/null | tail -n 80 >&2 || true
                else
                    tail -n 80 "$logfile" 2>/dev/null >&2 || true
                fi
            done
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

build_system_tarball() {
    local output
    output="$(guix_run_with_retries system image -v 2 --save-provenance $keep_failed --fallback -L "$script_dir" \
        --system=i686-linux --image-type=tarball "$script_dir/v86-os.scm")"
    echo "$output" | grep -o '/gnu/store/[^[:space:]]*\.tar\.gz' | tail -n 1
}

build_dillo_native() {
    guix_run_with_retries build $keep_failed --fallback -L "$script_dir" --system=i686-linux \
        -e '(@ (gnu packages web-browsers) dillo)'
}

cross_compile_dillo() {
    local dillo_output
    dillo_output="$(guix_run_with_retries build $keep_failed --fallback -L "$script_dir" \
        --system=x86_64-linux --target=i686-linux-gnu \
        -e '((@ (patches) apply-patches) (@ (gnu packages web-browsers) dillo))')"
    echo "$dillo_output" | grep -o '/gnu/store/[^[:space:]]*dillo-[0-9.]*' | tail -n 1
}

fetch_dillo_sources() {
    guix_run_with_retries build -L "$script_dir" \
        -e '((@ (patches) all-transitive-sources) (list ((@ (patches) apply-patches) (@ (gnu packages web-browsers) dillo))))' || true
}

cross_compile_icecat() {
    local icecat_output
    icecat_output="$(guix_run_with_retries build $keep_failed --fallback -L "$script_dir" \
        --system=x86_64-linux --target=i686-linux-gnu \
        -e '((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat))')"
    echo "$icecat_output" | grep -o '/gnu/store/[^[:space:]]*icecat-[0-9.]*' | tail -n 1
}

fetch_icecat_sources() {
    guix_run_with_retries build -L "$script_dir" \
        -e '((@ (patches) all-transitive-sources) (list ((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat-minimal)) ((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat))))' || true
}

fetch_system_sources() {
    guix_run_with_retries build -L "$script_dir" \
        -e '((@ (patches) all-transitive-sources) ((@ (gnu system) operating-system-packages) (load "'"$script_dir"'/v86-os.scm")))' || true
}

case "$mode" in
    pull)
        start_guix_daemon
        echo "Updating Guix channels to latest..."
        guix_run_with_retries pull --url="https://codeberg.org/guix/guix.git" --substitute-urls="https://bordeaux.guix.gnu.org https://ci.guix.gnu.org"
        for key in /var/guix/profiles/per-user/root/current-profile/share/guix/*.pub \
                   /root/.config/guix/current/share/guix/*.pub \
                   /usr/local/share/guix/*.pub; do
            if [ -f "$key" ]; then
                guix archive --authorize < "$key" || true
            fi
        done
        echo "Guix pull complete."
        stop_guix_daemon
        ;;

    smoke-test-native)
        start_guix_daemon
        echo "Smoke testing native package resolution with GNU Hello..."
        guix_run_with_retries build $keep_failed --fallback --system=i686-linux hello
        echo "Native smoke test passed."
        stop_guix_daemon
        ;;

    smoke-test-cross)
        start_guix_daemon
        echo "Smoke testing cross-compilation toolchain with GNU Hello..."
        guix_run_with_retries build $keep_failed --fallback --system=x86_64-linux --target=i686-linux-gnu hello
        echo "Cross-compilation smoke test passed."
        stop_guix_daemon
        ;;

    build-dillo-native)
        start_guix_daemon
        echo "Building Dillo natively for i686-linux..."
        build_dillo_native
        echo "Native Dillo build complete."
        stop_guix_daemon
        ;;

    cross-dillo)
        start_guix_daemon
        echo "Cross-compiling Dillo from x86_64 for i686-linux-gnu..."
        dillo_store_path="$(cross_compile_dillo)"
        echo "Cross-compiled Dillo at: $dillo_store_path"
        echo "Fetching and realizing Dillo source closure..."
        fetch_dillo_sources
        stop_guix_daemon
        ;;

    cross-icecat)
        start_guix_daemon
        echo "Cross-compiling GNU Icecat from x86_64 for i686-linux-gnu..."
        icecat_store_path="$(cross_compile_icecat)"
        echo "Cross-compiled Icecat at: $icecat_store_path"
        echo "Fetching and realizing Icecat source closure..."
        fetch_icecat_sources
        stop_guix_daemon
        ;;

    prebuild-tarball)
        start_guix_daemon
        echo "Building Guix i686 system tarball image..."
        tarball_img="$(build_system_tarball)"
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
        dillo_store_path=""

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
            tarball_img="$(build_system_tarball)"
        else
            echo "Error: No prebuilt tarball found and 'guix' is not available." >&2
            exit 1
        fi

        if [ "$guix_available" -eq 1 ]; then
            if [ -n "${DISABLE_CROSS:-}" ] && [ "$DISABLE_CROSS" = "1" ]; then
                echo "Skipping browser cross-compilation (DISABLE_CROSS set)."
            else
                echo "Cross-compiling GNU Icecat from x86_64 for i686-linux-gnu..."
                icecat_store_path="$(cross_compile_icecat || true)"
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

        # if [ -n "${dillo_store_path:-}" ] && [ -d "$dillo_store_path" ]; then
        #     echo "Merging cross-compiled Dillo closure ($dillo_store_path) into rootfs..."
        #     dillo_closure="$(guix gc -R "$dillo_store_path")"
        #     mkdir -p "$tmp_rootfs_dir/gnu/store"
        #     for store_item in $dillo_closure; do
        #         if [ -e "$store_item" ]; then
        #             cp -a "$store_item" "$tmp_rootfs_dir/gnu/store/"
        #         fi
        #     done

        #     echo "Fetching and merging Dillo source closure into rootfs..."
        #     dillo_sources="$(fetch_dillo_sources)"
        #     for src_item in $dillo_sources; do
        #         if [ -n "$src_item" ] && [ -e "$src_item" ]; then
        #             src_closure="$(guix gc -R "$src_item")"
        #             for item in $src_closure; do
        #                 cp -a "$item" "$tmp_rootfs_dir/gnu/store/"
        #             done
        #         fi
        #     done

        #     sys_profile="$(find "$tmp_rootfs_dir/gnu/store" -maxdepth 1 -name "*-profile" | head -n 1 || true)"
        #     if [ -n "$sys_profile" ] && [ -d "$sys_profile/bin" ]; then
        #         ln -sf "$dillo_store_path/bin/dillo" "$sys_profile/bin/dillo"
        #     fi
        #     mkdir -p "$tmp_rootfs_dir/usr/local/bin"
        #     ln -sf "$dillo_store_path/bin/dillo" "$tmp_rootfs_dir/usr/local/bin/dillo"
        #     echo "Successfully merged Dillo into Guix rootfs!"
        # fi

        if [ -n "${icecat_store_path:-}" ] && [ -d "$icecat_store_path" ]; then
            echo "Merging cross-compiled Icecat closure ($icecat_store_path) into rootfs..."
            icecat_closure="$(guix gc -R "$icecat_store_path")"
            mkdir -p "$tmp_rootfs_dir/gnu/store"
            for store_item in $icecat_closure; do
                if [ -e "$store_item" ]; then
                    cp -a "$store_item" "$tmp_rootfs_dir/gnu/store/"
                fi
            done

            echo "Fetching and merging Icecat source closure into rootfs..."
            icecat_sources="$(fetch_icecat_sources)"
            for src_item in $icecat_sources; do
                if [ -n "$src_item" ] && [ -e "$src_item" ]; then
                    src_closure="$(guix gc -R "$src_item")"
                    for item in $src_closure; do
                        cp -a "$item" "$tmp_rootfs_dir/gnu/store/"
                    done
                fi
            done

            echo "Fetching and merging base system source closure into rootfs..."
            system_sources="$(fetch_system_sources)"
            for src_item in $system_sources; do
                if [ -n "$src_item" ] && [ -e "$src_item" ]; then
                    src_closure="$(guix gc -R "$src_item")"
                    for item in $src_closure; do
                        cp -a "$item" "$tmp_rootfs_dir/gnu/store/"
                    done
                fi
            done

            sys_profile="$(find "$tmp_rootfs_dir/gnu/store" -maxdepth 1 -name "*-profile" | head -n 1 || true)"
            if [ -n "$sys_profile" ] && [ -d "$sys_profile/bin" ]; then
                ln -sf "$icecat_store_path/bin/icecat" "$sys_profile/bin/icecat"
            fi
            mkdir -p "$tmp_rootfs_dir/usr/local/bin"
            ln -sf "$icecat_store_path/bin/icecat" "$tmp_rootfs_dir/usr/local/bin/icecat"
            echo "Successfully merged Icecat and source code into Guix rootfs!"
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
