#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Collective Toolbox Developers

# Run Guix garbage collection safely within the devcontainer, protecting
# all essential cross-compiled packages and v86 image components while
# reclaiming disk space from temporary build artifacts and superseded items.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Ensure GC roots directory for ctoolbox exists
mkdir -p /var/guix/gcroots/ctoolbox

# Register GC roots for key ctoolbox artifacts if present in /gnu/store.
# Only protect actual directory outputs or source tarballs/checkouts, never
# temporary .drv, -builder scripts, lockfiles, or partial build trees.
protect_store_item() {
    local target="$1"
    local name="$2"
    if [ -e "${target}" ]; then
        case "${target}" in
            *-builder|*.drv|*.lock|*.tmp|*-checkout-builder)
                return 0
                ;;
        esac
        ln -sf "${target}" "/var/guix/gcroots/ctoolbox/${name}"
    fi
}

echo "Registering GC roots for essential workspace packages..."

# Protect all current system profiles
for p in /gnu/store/*-profile; do
    protect_store_item "${p}" "$(basename "${p}")"
done

# Protect final output directories for Icecat, Dillo, Mesa, and LLVM
for ic in /gnu/store/*-icecat-[0-9]*; do
    protect_store_item "${ic}" "$(basename "${ic}")"
done

for icm in /gnu/store/*-icecat-minimal-[0-9]*; do
    protect_store_item "${icm}" "$(basename "${icm}")"
done

for d in /gnu/store/*-dillo-3*; do
    protect_store_item "${d}" "$(basename "${d}")"
done

for m in /gnu/store/*-mesa-26*; do
    protect_store_item "${m}" "$(basename "${m}")"
done

for l in /gnu/store/*-llvm-21*; do
    protect_store_item "${l}" "$(basename "${l}")"
done

# Protect source archives and checkouts
for s in /gnu/store/*.tar.* /gnu/store/*.tgz /gnu/store/*-checkout; do
    protect_store_item "${s}" "$(basename "${s}")"
done

# Clean up failed/interrupted build working directories in /tmp
echo "Cleaning up leftover build working directories in /tmp..."
for build_dir in /tmp/guix-build-*.drv-*; do
    if [ -d "${build_dir}" ]; then
        rm -rf "${build_dir}"
    fi
done

# Clean up broken symlinks in roots if any
find /var/guix/gcroots/ctoolbox -xtype l -delete

# Ensure directories and permissions for daemon
mkdir -p /var/log/guix/drvs /var/guix /var/guix/daemon-socket
chown -R root:guixbuild /var/guix /var/log/guix /gnu/store
chmod 755 /gnu/store /var/guix

# Check if guix-daemon is responding
daemon_running=0
if [ -e /var/guix/daemon-socket/socket ]; then
    if guix gc --list-dead >/dev/null 2>&1; then
        daemon_running=1
    else
        # Remove stale socket
        rm /var/guix/daemon-socket/socket
    fi
fi

started_daemon_pid=""
if [ "${daemon_running}" -eq 0 ]; then
    echo "Starting guix-daemon in background..."
    nopersonality_preload=""
    if [ -f /gnu/store/libctb_nopersonality.so ]; then
        nopersonality_preload="LD_PRELOAD=/gnu/store/libctb_nopersonality.so"
    elif [ -f /usr/lib/libctb_nopersonality.so ]; then
        nopersonality_preload="LD_PRELOAD=/usr/lib/libctb_nopersonality.so"
    fi

    if [ -n "${nopersonality_preload}" ]; then
        env "${nopersonality_preload}" guix-daemon \
            --build-users-group=guixbuild \
            --disable-chroot \
            </dev/null >/var/log/guix-daemon.log 2>&1 &
    else
        guix-daemon \
            --build-users-group=guixbuild \
            --disable-chroot \
            </dev/null >/var/log/guix-daemon.log 2>&1 &
    fi
    started_daemon_pid=$!

    # Wait for socket to become ready
    max_wait=15
    while [ "${max_wait}" -gt 0 ]; do
        if [ -e /var/guix/daemon-socket/socket ]; then
            break
        fi
        sleep 1
        max_wait=$((max_wait - 1))
    done
fi

cleanup() {
    if [ -n "${started_daemon_pid}" ]; then
        echo "Stopping temporarily started guix-daemon..."
        kill "${started_daemon_pid}" 2>/dev/null || true
        wait "${started_daemon_pid}" 2>/dev/null || true
        if [ -e /var/guix/daemon-socket/socket ]; then
            rm /var/guix/daemon-socket/socket
        fi
    fi
}
trap cleanup EXIT

echo "Current disk usage before GC:"
df -h /gnu/store

echo "Running guix gc ${*:-}..."
guix gc "$@"

echo "Optimizing store links..."
guix gc --optimize 2>/dev/null || true

echo "Disk usage after GC:"
df -h /gnu/store
