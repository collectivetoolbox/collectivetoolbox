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
# install-gui-tools.sh: Install virtual framebuffer (Xvfb), X11 window manager,
# and tools to capture and interact with GUI applications.
#

set -euo pipefail

cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/../.." || exit 1

IFS=$'\n\t'

echo "=== Installing GUI environment and automation tools ==="

export DEBIAN_FRONTEND=noninteractive

dpkg --add-architecture i386 || true
apt-get update

apt-get install -y --no-install-recommends \
    xvfb \
    xauth \
    dbus-x11 \
    openbox \
    fluxbox \
    xdotool \
    x11-utils \
    x11-xserver-utils \
    x11-apps \
    scrot \
    maim \
    imagemagick \
    xclip \
    xsel \
    ffmpeg \
    fonts-dejavu-core \
    fonts-liberation

# Clean up apt caches
apt-get clean
if [ -d /var/lib/apt/lists ]; then
    find /var/lib/apt/lists -maxdepth 1 -mindepth 1 -exec rm -r {} + || true
fi

# Set system-wide default DISPLAY=:99
echo 'export DISPLAY="${DISPLAY:-:99}"' > /etc/profile.d/gui-display.sh
chmod 755 /etc/profile.d/gui-display.sh

# Install GUI helper scripts into /usr/local/bin
echo "=== Installing helper utilities into /usr/local/bin ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_SCRIPTS="$(cd "${SCRIPT_DIR}/.." 2>/dev/null && pwd || echo "")"

install_helper() {
    local name="$1"
    local repo_file="${REPO_SCRIPTS}/${name}"
    local target="/usr/local/bin/${name}"

    if [ -f "${repo_file}" ]; then
        if [ -e "${target}" ]; then
            rm "${target}"
        fi
        cp "${repo_file}" "${target}"
        chmod 755 "${target}"
    fi
}

for tool in gui-server gui-run gui-screenshot gui-record gui-inspect; do
    install_helper "$tool"
done

# If repo scripts were not present (e.g. running from /tmp during Docker build), create them:
if [ ! -f /usr/local/bin/gui-server ]; then
cat <<'EOF' > /usr/local/bin/gui-server
#!/usr/bin/env bash
set -euo pipefail

DISPLAY_NUM="${GUI_DISPLAY:-${DISPLAY:-:99}}"
RESOLUTION="${GUI_RESOLUTION:-1280x800x24}"
DISPLAY_ID="${DISPLAY_NUM#:}"
PID_FILE_XVFB="/tmp/.xvfb_${DISPLAY_ID}.pid"
PID_FILE_WM="/tmp/.wm_${DISPLAY_ID}.pid"
LOG_XVFB="/tmp/xvfb_${DISPLAY_ID}.log"
LOG_WM="/tmp/wm_${DISPLAY_ID}.log"

is_process_running() {
    local pid_file="$1"
    local process_pattern="${2:-}"
    if [ -f "$pid_file" ]; then
        local pid
        pid="$(cat "$pid_file" 2>/dev/null || echo "")"
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            return 0
        fi
    fi
    if [ -n "$process_pattern" ]; then
        if pgrep -f "$process_pattern" >/dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

start_server() {
    if is_process_running "$PID_FILE_XVFB" "Xvfb ${DISPLAY_NUM}"; then
        echo "Xvfb is already running on ${DISPLAY_NUM}"
    else
        echo "Starting Xvfb on ${DISPLAY_NUM} (${RESOLUTION})..."
        local socket_file="/tmp/.X11-unix/X${DISPLAY_ID}"
        if [ -e "$socket_file" ] && ! is_process_running "$PID_FILE_XVFB" "Xvfb ${DISPLAY_NUM}"; then
            if [ -S "$socket_file" ] || [ -f "$socket_file" ]; then
                rm "$socket_file"
            fi
        fi
        local lock_file="/tmp/.X${DISPLAY_ID}-lock"
        if [ -e "$lock_file" ] && ! is_process_running "$PID_FILE_XVFB" "Xvfb ${DISPLAY_NUM}"; then
            if [ -f "$lock_file" ]; then
                rm "$lock_file"
            fi
        fi

        mkdir -p /tmp/.X11-unix
        chmod 1777 /tmp/.X11-unix

        setsid -w Xvfb "${DISPLAY_NUM}" -screen 0 "${RESOLUTION}" -ac +extension GLX +render -noreset > "${LOG_XVFB}" 2>&1 < /dev/null &
        local launcher_pid=$!
        sleep 0.2
        local xvfb_pid
        xvfb_pid="$(pgrep -n -f "Xvfb ${DISPLAY_NUM}" 2>/dev/null || echo "$launcher_pid")"
        echo "$xvfb_pid" > "$PID_FILE_XVFB"

        local attempts=0
        local max_attempts=40
        local ready=0
        while [ "$attempts" -lt "$max_attempts" ]; do
            if DISPLAY="${DISPLAY_NUM}" xdpyinfo >/dev/null 2>&1; then
                ready=1
                break
            fi
            sleep 0.1
            attempts=$((attempts + 1))
        done

        if [ "$ready" -ne 1 ]; then
            echo "Error: Failed to start Xvfb on ${DISPLAY_NUM}. Log contents (${LOG_XVFB}):" >&2
            if [ -f "${LOG_XVFB}" ]; then
                cat "${LOG_XVFB}" >&2
            fi
            exit 1
        fi
        echo "Xvfb started successfully on ${DISPLAY_NUM} (PID: ${xvfb_pid})"
    fi

    if is_process_running "$PID_FILE_WM" "(openbox|fluxbox)"; then
        echo "Window manager is already running on ${DISPLAY_NUM}"
    else
        local wm_bin=""
        if command -v openbox >/dev/null 2>&1; then
            wm_bin="openbox"
        elif command -v fluxbox >/dev/null 2>&1; then
            wm_bin="fluxbox"
        fi

        if [ -n "$wm_bin" ]; then
            echo "Starting window manager (${wm_bin}) on ${DISPLAY_NUM}..."
            DISPLAY="${DISPLAY_NUM}" setsid -w "$wm_bin" > "${LOG_WM}" 2>&1 < /dev/null &
            local wm_launcher_pid=$!
            sleep 0.2
            local wm_pid
            wm_pid="$(pgrep -n -f "^${wm_bin}" 2>/dev/null || echo "$wm_launcher_pid")"
            echo "$wm_pid" > "$PID_FILE_WM"
            echo "Window manager (${wm_bin}) started (PID: ${wm_pid})"
        fi
    fi
}

stop_server() {
    echo "Stopping GUI environment on ${DISPLAY_NUM}..."
    if [ -f "$PID_FILE_WM" ]; then
        local wm_pid
        wm_pid="$(cat "$PID_FILE_WM" 2>/dev/null || echo "")"
        if [ -n "$wm_pid" ]; then
            kill "$wm_pid" 2>/dev/null || true
        fi
        rm "$PID_FILE_WM"
    fi
    pkill -f "^(openbox|fluxbox)" 2>/dev/null || true
    echo "Window manager stopped."

    if [ -f "$PID_FILE_XVFB" ]; then
        local xvfb_pid
        xvfb_pid="$(cat "$PID_FILE_XVFB" 2>/dev/null || echo "")"
        if [ -n "$xvfb_pid" ]; then
            kill "$xvfb_pid" 2>/dev/null || true
        fi
        rm "$PID_FILE_XVFB"
    fi
    pkill -f "Xvfb ${DISPLAY_NUM}" 2>/dev/null || true
    echo "Xvfb stopped."

    local lock_file="/tmp/.X${DISPLAY_ID}-lock"
    if [ -f "$lock_file" ]; then
        rm "$lock_file"
    fi
    local socket_file="/tmp/.X11-unix/X${DISPLAY_ID}"
    if [ -e "$socket_file" ]; then
        if [ -S "$socket_file" ] || [ -f "$socket_file" ]; then
            rm "$socket_file"
        fi
    fi
}

status_server() {
    local running=1
    if is_process_running "$PID_FILE_XVFB" "Xvfb ${DISPLAY_NUM}"; then
        local xvfb_pid
        xvfb_pid="$(pgrep -n -f "Xvfb ${DISPLAY_NUM}" 2>/dev/null || (cat "$PID_FILE_XVFB" 2>/dev/null || echo "unknown"))"
        echo "Xvfb: RUNNING on ${DISPLAY_NUM} (PID: ${xvfb_pid})"
        if DISPLAY="${DISPLAY_NUM}" xdpyinfo >/dev/null 2>&1; then
            DISPLAY="${DISPLAY_NUM}" xdpyinfo | grep "dimensions:" | sed 's/^[ \t]*/  /'
        fi
    else
        echo "Xvfb: NOT RUNNING on ${DISPLAY_NUM}"
        running=0
    fi

    if is_process_running "$PID_FILE_WM" "(openbox|fluxbox)"; then
        local wm_pid
        wm_pid="$(pgrep -n -f "^(openbox|fluxbox)" 2>/dev/null || (cat "$PID_FILE_WM" 2>/dev/null || echo "unknown"))"
        echo "Window Manager: RUNNING (PID: ${wm_pid})"
    else
        echo "Window Manager: NOT RUNNING"
    fi

    if [ "$running" -eq 1 ]; then
        echo "Active clients:"
        DISPLAY="${DISPLAY_NUM}" xlsclients -l 2>/dev/null || true
        return 0
    else
        return 1
    fi
}

case "${1:-status}" in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    restart)
        stop_server
        sleep 1
        start_server
        ;;
    status)
        status_server
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}" >&2
        exit 1
        ;;
esac
EOF
chmod 755 /usr/local/bin/gui-server
fi

if [ ! -f /usr/local/bin/gui-run ]; then
cat <<'EOF' > /usr/local/bin/gui-run
#!/usr/bin/env bash
set -euo pipefail

DISPLAY_NUM="${GUI_DISPLAY:-${DISPLAY:-:99}}"

if ! DISPLAY="${DISPLAY_NUM}" xdpyinfo >/dev/null 2>&1; then
    gui_server_cmd="gui-server"
    if ! command -v "$gui_server_cmd" >/dev/null 2>&1; then
        script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        if [ -x "${script_dir}/gui-server" ]; then
            gui_server_cmd="${script_dir}/gui-server"
        fi
    fi
    "$gui_server_cmd" start
fi

export DISPLAY="${DISPLAY_NUM}"
exec "$@"
EOF
chmod 755 /usr/local/bin/gui-run
fi

if [ ! -f /usr/local/bin/gui-screenshot ]; then
cat <<'EOF' > /usr/local/bin/gui-screenshot
#!/usr/bin/env bash
set -euo pipefail

DISPLAY_NUM="${GUI_DISPLAY:-${DISPLAY:-:99}}"
export DISPLAY="${DISPLAY_NUM}"

OUTPUT="${1:-}"
TARGET="${2:-}"

if [ -z "$OUTPUT" ]; then
    TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
    OUTPUT="/tmp/screenshot_${TIMESTAMP}.png"
fi

DIRNAME="$(dirname "$OUTPUT")"
if [ ! -d "$DIRNAME" ]; then
    mkdir -p "$DIRNAME"
fi

WIN_ID=""
if [ -n "$TARGET" ]; then
    if [[ "$TARGET" =~ ^0x[0-9a-fA-F]+$ ]] || [[ "$TARGET" =~ ^[0-9]+$ ]]; then
        WIN_ID="$TARGET"
    else
        WIN_ID="$(xdotool search --name "$TARGET" 2>/dev/null | head -n 1 || true)"
        if [ -z "$WIN_ID" ]; then
            WIN_ID="$(xdotool search --class "$TARGET" 2>/dev/null | head -n 1 || true)"
        fi
    fi
fi

if [ -n "$WIN_ID" ]; then
    echo "Capturing window ${WIN_ID} to ${OUTPUT}..."
    if command -v maim >/dev/null 2>&1; then
        maim -i "$WIN_ID" "$OUTPUT"
    elif command -v import >/dev/null 2>&1; then
        import -window "$WIN_ID" "$OUTPUT"
    elif command -v scrot >/dev/null 2>&1; then
        xdotool windowactivate --sync "$WIN_ID" 2>/dev/null || true
        scrot -u "$OUTPUT"
    else
        echo "Error: No screenshot tool available (maim, import, scrot)" >&2
        exit 1
    fi
else
    echo "Capturing full screen (${DISPLAY_NUM}) to ${OUTPUT}..."
    if command -v maim >/dev/null 2>&1; then
        maim "$OUTPUT"
    elif command -v scrot >/dev/null 2>&1; then
        scrot "$OUTPUT"
    elif command -v import >/dev/null 2>&1; then
        import -window root "$OUTPUT"
    else
        echo "Error: No screenshot tool available (maim, scrot, import)" >&2
        exit 1
    fi
fi

if [ -f "$OUTPUT" ]; then
    FILE_SIZE="$(stat -c%s "$OUTPUT" 2>/dev/null || stat -f%z "$OUTPUT" 2>/dev/null || echo "unknown")"
    echo "Screenshot saved: ${OUTPUT} (${FILE_SIZE} bytes)"
    if command -v identify >/dev/null 2>&1; then
        identify "$OUTPUT"
    fi
else
    echo "Error: Failed to create ${OUTPUT}" >&2
    exit 1
fi
EOF
chmod 755 /usr/local/bin/gui-screenshot
fi

if [ ! -f /usr/local/bin/gui-record ]; then
cat <<'EOF' > /usr/local/bin/gui-record
#!/usr/bin/env bash
set -euo pipefail

DISPLAY_NUM="${GUI_DISPLAY:-${DISPLAY:-:99}}"
DISPLAY_ID="${DISPLAY_NUM#:}"
PID_FILE="/tmp/.gui-record_${DISPLAY_ID}.pid"

case "${1:-}" in
    start)
        OUTPUT="${2:-/tmp/recording_$(date +%Y%m%d_%H%M%S).mp4}"
        FPS="${3:-30}"

        if [ -f "$PID_FILE" ]; then
            OLD_PID="$(cat "$PID_FILE" 2>/dev/null || echo "")"
            if [ -n "$OLD_PID" ] && kill -0 "$OLD_PID" 2>/dev/null; then
                echo "Recording is already in progress (PID: $OLD_PID)"
                exit 0
            fi
        fi

        DIRNAME="$(dirname "$OUTPUT")"
        if [ ! -d "$DIRNAME" ]; then
            mkdir -p "$DIRNAME"
        fi

        RES="$(DISPLAY="${DISPLAY_NUM}" xdpyinfo 2>/dev/null | grep "dimensions:" | awk '{print $2}' || echo "1280x800")"
        echo "Starting recording of ${DISPLAY_NUM} (${RES}) at ${FPS} fps to ${OUTPUT}..."

        ffmpeg -y -f x11grab -draw_mouse 1 -video_size "${RES}" -framerate "${FPS}" -i "${DISPLAY_NUM}.0" \
            -c:v libx264 -preset ultrafast -pix_fmt yuv420p "${OUTPUT}" > /tmp/gui-record.log 2>&1 &
        REC_PID=$!
        echo "$REC_PID" > "$PID_FILE"
        echo "Recording started (PID: ${REC_PID}, File: ${OUTPUT})"
        ;;
    stop)
        if [ -f "$PID_FILE" ]; then
            REC_PID="$(cat "$PID_FILE" 2>/dev/null || echo "")"
            if [ -n "$REC_PID" ] && kill -0 "$REC_PID" 2>/dev/null; then
                echo "Stopping recording (PID: ${REC_PID})..."
                kill -INT "$REC_PID" 2>/dev/null || true
                sleep 1
            fi
            if [ -f "$PID_FILE" ]; then
                rm "$PID_FILE"
            fi
            echo "Recording stopped."
        else
            echo "No active recording found."
        fi
        ;;
    *)
        echo "Usage: gui-record start [output.mp4] [fps] | gui-record stop" >&2
        exit 1
        ;;
esac
EOF
chmod 755 /usr/local/bin/gui-record
fi

if [ ! -f /usr/local/bin/gui-inspect ]; then
cat <<'EOF' > /usr/local/bin/gui-inspect
#!/usr/bin/env bash
set -euo pipefail

DISPLAY_NUM="${GUI_DISPLAY:-${DISPLAY:-:99}}"
export DISPLAY="${DISPLAY_NUM}"

echo "=== GUI Display Inspection (${DISPLAY_NUM}) ==="
if ! xdpyinfo >/dev/null 2>&1; then
    echo "Error: Display ${DISPLAY_NUM} is not available." >&2
    exit 1
fi

echo "--- Screen Dimensions & Depth ---"
xdpyinfo | grep -E "(dimensions:|depth of root window:)" | sed 's/^[ \t]*/  /'

echo "--- Mouse Pointer Location ---"
xdotool getmouselocation --shell 2>/dev/null || true

echo "--- Active Window ---"
ACTIVE_WIN="$(xdotool getactivewindow 2>/dev/null || echo "")"
if [ -n "$ACTIVE_WIN" ]; then
    echo "  Window ID: ${ACTIVE_WIN}"
    WIN_NAME="$(xdotool getwindowname "$ACTIVE_WIN" 2>/dev/null || echo "<unnamed>")"
    echo "  Window Name: ${WIN_NAME}"
    xdotool getwindowgeometry "$ACTIVE_WIN" 2>/dev/null | sed 's/^[ \t]*/  /' || true
else
    echo "  No active window."
fi

echo "--- All Windows ---"
xdotool search --onlyvisible --name ".*" 2>/dev/null | while read -r win; do
    name="$(xdotool getwindowname "$win" 2>/dev/null || echo "<unnamed>")"
    geom="$(xdotool getwindowgeometry "$win" 2>/dev/null | grep "Geometry:" | awk '{print $2}' || echo "?")"
    pos="$(xdotool getwindowgeometry "$win" 2>/dev/null | grep "Position:" | awk '{print $2}' || echo "?")"
    echo "  ID: ${win} | Title: \"${name}\" | Pos: ${pos} | Size: ${geom}"
done
EOF
chmod 755 /usr/local/bin/gui-inspect
fi

echo "=== GUI tools installation complete ==="
