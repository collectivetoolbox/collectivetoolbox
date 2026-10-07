#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# This file is part of Collective Toolbox, a database and document workspace and utilities.
# Copyright (C) 2026 Collective Toolbox Developers
# Contact: info@collectivetoolbox.com
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option) any
# later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along
# with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# Shared utilities for QEMU VM management, snapshot control, fixture packaging,
# output streaming, and CI logging.
set -euo pipefail

cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/.." || exit 1

remove_if_exists() {
    for path in "$@"; do
        if [[ -e "$path" || -L "$path" || -S "$path" ]]; then
            rm -r "$path"
        fi
    done
}

vm_is_running() {
    local pid_file="$1"
    if [[ -f "$pid_file" ]]; then
        local pid
        pid="$(cat "$pid_file")"
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

vm_cmd_stop() {
    local pid_file="$1"
    local monitor_sock="$2"
    local vnc_sock="$3"
    local force="${4:-false}"
    local vm_name="${5:-VM}"

    if ! vm_is_running "$pid_file"; then
        echo "$vm_name is not running."
        remove_if_exists "$pid_file" "$monitor_sock" "$vnc_sock"
        return 0
    fi

    local pid
    pid="$(cat "$pid_file")"

    if [[ "$force" == "true" ]]; then
        echo "Terminating $vm_name (PID $pid)..."
        kill -9 "$pid" 2>/dev/null || true
    else
        echo "Sending ACPI shutdown to $vm_name (PID $pid)..."
        vm_cmd_monitor "$monitor_sock" "system_powerdown" >/dev/null 2>&1 || kill -15 "$pid" 2>/dev/null || true
        for _ in $(seq 1 15); do
            if ! kill -0 "$pid" 2>/dev/null; then
                break
            fi
            sleep 1
        done
        if kill -0 "$pid" 2>/dev/null; then
            echo "$vm_name did not shut down gracefully; stopping forcefully..."
            kill -9 "$pid" 2>/dev/null || true
            while kill -0 "$pid" 2>/dev/null; do
                sleep 0.2
            done
            sleep 1
        fi
    fi

    remove_if_exists "$pid_file" "$monitor_sock" "$vnc_sock"
    echo "$vm_name stopped."
}

vm_cmd_status() {
    local pid_file="$1"
    local vm_name="${2:-VM}"
    if vm_is_running "$pid_file"; then
        local pid
        pid="$(cat "$pid_file")"
        echo "$vm_name is running (PID: $pid)"
        ps -o pid,user,%cpu,%mem,rss,cmd -p "$pid"
    else
        echo "$vm_name is not running."
    fi
}

vm_cmd_monitor() {
    local monitor_sock="$1"
    shift
    local cmd="$*"

    if [[ ! -S "$monitor_sock" ]]; then
        echo "Error: Monitor socket $monitor_sock not found or VM not running." >&2
        return 1
    fi

    python3 -c "
import socket, sys
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sys.argv[1])
s.recv(1024)
s.sendall(sys.argv[2].encode() + b'\n')
import time; time.sleep(0.3)
print(s.recv(4096).decode(errors='replace').strip())
s.close()
" "$monitor_sock" "$cmd"
}

vm_cmd_screenshot() {
    local monitor_sock="$1"
    local out_path="$2"
    local tmp_ppm="/tmp/vm-screen-tmp-$$.ppm"

    if [[ ! -S "$monitor_sock" ]]; then
        echo "Error: Monitor socket $monitor_sock not found or VM not running." >&2
        return 1
    fi

    vm_cmd_monitor "$monitor_sock" "screendump $tmp_ppm" >/dev/null
    sleep 0.5

    if [[ -f "$tmp_ppm" ]]; then
        ffmpeg -y -i "$tmp_ppm" "$out_path" 2>/dev/null || convert "$tmp_ppm" "$out_path"
        remove_if_exists "$tmp_ppm"
        echo "Screenshot saved to $out_path"
    else
        echo "Error: Failed to capture screen dump." >&2
        return 1
    fi
}

vm_cmd_send_keys() {
    local monitor_sock="$1"
    shift
    for key in "$@"; do
        vm_cmd_monitor "$monitor_sock" "sendkey $key" >/dev/null
        sleep 0.2
    done
}

vm_cmd_type() {
    local monitor_sock="$1"
    local text="$2"

    if [[ ! -S "$monitor_sock" ]]; then
        echo "Error: Monitor socket $monitor_sock not found or VM not running." >&2
        return 1
    fi

    python3 -c "
import socket, sys, time

monitor_sock = sys.argv[1]
text = sys.argv[2]

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(monitor_sock)
s.recv(1024)

def send(key):
    s.sendall(f'sendkey {key} 20\n'.encode())
    time.sleep(0.04)

key_map = {
    ':': 'shift-semicolon',
    ';': 'semicolon',
    '\\\\': 'backslash',
    '/': 'slash',
    '.': 'dot',
    ',': 'comma',
    '-': 'minus',
    '_': 'shift-minus',
    '=': 'equal',
    '+': 'shift-equal',
    ' ': 'spc',
    '>': 'shift-dot',
    '<': 'shift-comma',
    '*': 'shift-8',
    '\"': 'shift-apostrophe',
    \"'\": 'apostrophe',
    '|': 'shift-backslash',
    '[': 'bracket_left',
    ']': 'bracket_right',
    '{': 'shift-bracket_left',
    '}': 'shift-bracket_right',
    '(': 'shift-9',
    ')': 'shift-0',
    '&': 'shift-7',
    '%': 'shift-5',
    '$': 'shift-4',
    '#': 'shift-3',
    '@': 'shift-2',
    '!': 'shift-1',
    '?': 'shift-slash',
    '~': 'shift-grave_accent',
    '\`': 'grave_accent',
    '\n': 'ret',
}

for ch in text:
    if ch in key_map:
        send(key_map[ch])
    elif ch.isupper():
        send(f'shift-{ch.lower()}')
    elif ch.isdigit() or ch.islower():
        send(ch)
    else:
        send(ch)

s.close()
" "$monitor_sock" "$text"
}

vm_cmd_snapshot() {
    local monitor_sock="$1"
    local pid_file="$2"
    local image="$3"
    local subcmd="${4:-list}"
    local name="${5:-}"

    case "$subcmd" in
        list)
            qemu-img snapshot -l "$image"
            ;;
        create)
            if [[ -z "$name" ]]; then
                echo "Usage: snapshot create <name>" >&2
                return 1
            fi
            if vm_is_running "$pid_file"; then
                echo "VM is currently running. Creating live snapshot '$name'..."
                vm_cmd_monitor "$monitor_sock" "savevm $name"
            else
                echo "Creating disk snapshot '$name' on $image..."
                qemu-img snapshot -c "$name" "$image"
            fi
            echo "Snapshot '$name' created."
            ;;
        revert)
            if [[ -z "$name" ]]; then
                echo "Usage: snapshot revert <name>" >&2
                return 1
            fi
            if vm_is_running "$pid_file"; then
                echo "Stopping VM before reverting snapshot..."
                vm_cmd_stop "$pid_file" "$monitor_sock" "" "false" "VM"
            fi
            echo "Reverting $image to snapshot '$name'..."
            qemu-img snapshot -a "$name" "$image"
            echo "Successfully reverted to snapshot '$name'."
            ;;
        delete)
            if [[ -z "$name" ]]; then
                echo "Usage: snapshot delete <name>" >&2
                return 1
            fi
            echo "Deleting snapshot '$name' from $image..."
            qemu-img snapshot -d "$name" "$image"
            echo "Snapshot '$name' deleted."
            ;;
        *)
            echo "Unknown snapshot subcommand: $subcmd" >&2
            echo "Supported: list, create, revert, delete" >&2
            return 1
            ;;
    esac
}

vm_package_fixtures() {
    local exchange_dir="$1"
    local repo_root="$2"

    echo "Packaging repository fixtures..."
    (cd "$repo_root" && find src -type d -name "fixtures" -print0 | tar --null -cf - --files-from=-) | (cd "$exchange_dir" && tar -xf -)

    echo "Packaging workspace markers, dcdata, and kaitai fixtures..."
    touch "$exchange_dir/.ctoolbox_workspace_root"
    cp "$repo_root/Cargo.toml" "$exchange_dir/Cargo.toml"
    mkdir -p "$exchange_dir/src/formats/dcdata"
    cp -r "$repo_root/src/formats/dcdata/data" "$exchange_dir/src/formats/dcdata/"
    mkdir -p "$exchange_dir/src/formats/kaitai"
    cp -r "$repo_root/src/formats/kaitai/kaitai_struct_tests" "$exchange_dir/src/formats/kaitai/"
    mkdir -p "$exchange_dir/scripts/guix"
    cp -r "$repo_root/scripts/guix/patches" "$exchange_dir/scripts/guix/"
    mkdir -p "$exchange_dir/src/storage/minimal" "$exchange_dir/storage/minimal"
    cp -r "$repo_root/src/storage/minimal/data" "$exchange_dir/src/storage/minimal/"
    cp -r "$repo_root/src/storage/minimal/data" "$exchange_dir/storage/minimal/"
    mkdir -p "$exchange_dir/src/utilities"
    cp "$repo_root/src/utilities/format_id.generated.rs" "$exchange_dir/src/utilities/"

    echo "Packaging TypeScript bootstrap inputs..."
    mkdir -p "$exchange_dir/vendor/TypeScript" "$exchange_dir/vendor/DefinitelyTyped-partial" "$exchange_dir/vendor/undici"
    cp -r "$repo_root/vendor/TypeScript/src" "$repo_root/vendor/TypeScript/scripts" "$exchange_dir/vendor/TypeScript/"
    cp -r "$repo_root/vendor/DefinitelyTyped-partial/node" "$repo_root/vendor/DefinitelyTyped-partial/deno" "$exchange_dir/vendor/DefinitelyTyped-partial/"
    cp -r "$repo_root/vendor/undici/types" "$exchange_dir/vendor/undici/"

    if [[ -f "$repo_root/built/ctoolbox.rsrc" ]]; then
        echo "Packaging runtime resource bundle (built/ctoolbox.rsrc)..."
        mkdir -p "$exchange_dir/built" "$exchange_dir/tests"
        cp "$repo_root/built/ctoolbox.rsrc" "$exchange_dir/built/ctoolbox.rsrc"
        cp "$repo_root/built/ctoolbox.rsrc" "$exchange_dir/tests/ctoolbox.rsrc"
        cp "$repo_root/built/ctoolbox.rsrc" "$exchange_dir/ctoolbox.rsrc"
    fi

    if [[ -f "$repo_root/vendor/v86_images/guix/guix-fs.json" ]]; then
        echo "Packaging vendor/v86_images/guix/guix-fs.json for v86 tests..."
        mkdir -p "$exchange_dir/vendor/v86_images/guix"
        cp "$repo_root/vendor/v86_images/guix/guix-fs.json" "$exchange_dir/vendor/v86_images/guix/"
    fi

    if [[ -d "$repo_root/vendor/v86/gen" ]]; then
        echo "Packaging vendor/v86/gen for v86 tests..."
        mkdir -p "$exchange_dir/vendor/v86"
        cp -r "$repo_root/vendor/v86/gen" "$exchange_dir/vendor/v86/"
    fi

}

vm_wait_for_marker() {
    python3 -c '
import sys, time

deadline = time.monotonic() + float(sys.argv[4])
pending = ""
with open(sys.argv[1], "r", errors="replace") as serial:
    serial.seek(int(sys.argv[2]))
    while True:
        line = serial.readline()
        if line:
            pending += line
            if pending.endswith("\n"):
                if pending.strip() == sys.argv[3]:
                    sys.exit(0)
                pending = ""
        elif time.monotonic() >= deadline:
            sys.exit(1)
        else:
            time.sleep(0.2)
' "$1" "$2" "$3" "$4"
}

vm_stream_suite_output() {
    local log_path="$1"
    local start_offset="$2"
    local timeout="$3"
    local suite_log_path="$4"
    local os_title="$5"

    python3 -c "
import sys, time, re

log_path = sys.argv[1]
start_offset = int(sys.argv[2])
timeout = int(sys.argv[3])
suite_log_path = sys.argv[4] if len(sys.argv) > 4 else None
os_title = sys.argv[5] if len(sys.argv) > 5 else 'VM'
expected_total = int(sys.argv[6]) if len(sys.argv) > 6 and sys.argv[6] else None

suite_log_file = None
if suite_log_path:
    try:
        suite_log_file = open(suite_log_path, 'w', errors='replace', buffering=1)
    except Exception as e:
        sys.stderr.write(f'Warning: Could not open {suite_log_path}: {e}\n')

start_time = time.time()
last_output_time = start_time
last_heartbeat_time = start_time
started = False
suite_total = 0
suite_passed = 0
suite_failed = 0
failing_binaries = []
passing_binaries = []
current_binary = ''
pending_line = ''

def write_out(text):
    sys.stdout.write(text)
    sys.stdout.flush()
    if suite_log_file:
        suite_log_file.write(text)
        suite_log_file.flush()

with open(log_path, 'r', errors='replace') as f:
    f.seek(start_offset)
    while True:
        line = f.readline()
        if line:
            last_output_time = time.time()
            pending_line += line
            if not pending_line.endswith('\n'):
                continue
            line = pending_line
            pending_line = ''
            clean = line.strip()
            if clean == '---SUITE_START---':
                started = True
                write_out(f'>>> {os_title} VM Test Suite Started <<<\n')
                continue
            if '---SUITE_END:' in clean:
                m = re.fullmatch(r'---SUITE_END:TOTAL=(\d+),PASSED=(\d+),FAILED=(\d+)---', clean)
                if not started or not m:
                    write_out('\n[ERROR] Invalid VM suite completion marker or missing startup marker.\n')
                    sys.exit(1)
                suite_total = int(m.group(1))
                suite_passed = int(m.group(2))
                suite_failed = int(m.group(3))
                if (suite_total <= 0 or suite_total != suite_passed + suite_failed
                    or suite_passed != len(passing_binaries)
                    or suite_failed != len(failing_binaries)
                    or (expected_total is not None and suite_total != expected_total)):
                    write_out('\n[ERROR] VM suite counts do not match executed or packaged binaries.\n')
                    sys.exit(1)
                break
            if started:
                m_run = re.search(r'=== RUN (\S+) ===', clean)
                if m_run:
                    current_binary = m_run.group(1)
                m_pass = re.search(r'=== PASS (\S+) ===', clean)
                if m_pass:
                    passing_binaries.append(m_pass.group(1))
                m_fail = re.search(r'=== FAIL (\S+)(?: \[EXIT:(-?\d+)\])?', clean)
                if m_fail:
                    failing_binaries.append((m_fail.group(1), m_fail.group(2) or 'unknown'))

                write_out(line)
        else:
            now = time.time()
            if not started and now - start_time > min(timeout, 300):
                write_out('\n[TIMEOUT] VM launcher did not produce the suite startup marker.\n')
                sys.exit(124)
            if now - start_time > timeout:
                msg = f'\n[TIMEOUT] Test suite exceeded {timeout} seconds in {os_title} VM.\n'
                write_out(msg)
                if suite_log_file:
                    suite_log_file.close()
                sys.exit(124)
            if started and now - last_heartbeat_time >= 30:
                elapsed_min = int((now - start_time) / 60)
                elapsed_sec = int((now - start_time) % 60)
                silence_sec = int(now - last_output_time)
                hb = f'[Heartbeat {elapsed_min:02d}:{elapsed_sec:02d}] Active: {current_binary} (no output for {silence_sec}s | passed: {len(passing_binaries)}, failed: {len(failing_binaries)})\n'
                write_out(hb)
                last_heartbeat_time = now
            time.sleep(0.2)

summary = []
summary.append('\n' + '=' * 60 + '\n')
summary.append(f'             {os_title.upper()} VM TEST SUITE SUMMARY\n')
summary.append('=' * 60 + '\n')
if failing_binaries:
    summary.append(f'FAILED TEST BINARIES ({len(failing_binaries)}):\n')
    for name, code in failing_binaries:
        summary.append(f'  [X] {name} (exit code: {code})\n')
else:
    summary.append('NO TEST FAILURES DETECTED!\n')

summary.append(f'PASSED TEST BINARIES: {len(passing_binaries)}\n')
summary.append(f'TOTAL: {suite_total} | PASSED: {suite_passed} | FAILED: {suite_failed}\n')
summary.append('=' * 60 + '\n')

for line in summary:
    write_out(line)

if suite_log_file:
    suite_log_file.close()

sys.exit(0 if suite_failed == 0 and len(failing_binaries) == 0 else 1)
" "$log_path" "$start_offset" "$timeout" "$suite_log_path" "$os_title" "${6:-}"
}

vm_capture_failure_artifacts() {
    local monitor_sock="$1"
    local serial_log="$2"
    local out_dir="$3"
    local os_title="${4:-VM}"

    mkdir -p "$out_dir"
    local ts
    ts="$(date +%Y%m%d-%H%M%S)"
    local screen_path="$out_dir/vm-timeout-${ts}.png"
    echo "Capturing $os_title VM screen before shutdown to $screen_path..."
    vm_cmd_screenshot "$monitor_sock" "$screen_path" 2>/dev/null || true
    if [[ -f "$screen_path" ]]; then
        cp "$screen_path" "$out_dir/vm-timeout-latest.png" 2>/dev/null || true
    fi
    if [[ -f "$serial_log" ]]; then
        echo "Exporting serial error log to $out_dir/vm-timeout-serial.log..."
        tail -n 1000 "$serial_log" > "$out_dir/vm-timeout-serial.log" 2>/dev/null || true
    fi
}
