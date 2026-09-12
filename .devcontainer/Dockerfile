FROM ghcr.io/collectivetoolbox/collectivetoolbox-2026-sept-5-2

# Install virtual framebuffer, X server, window manager, and GUI capture/interaction tools
COPY scripts/docker/install-gui-tools.sh /tmp/install-gui-tools.sh
RUN set -eux; \
    chmod +x /tmp/install-gui-tools.sh; \
    /tmp/install-gui-tools.sh; \
    if [ -f /tmp/install-gui-tools.sh ]; then \
        rm /tmp/install-gui-tools.sh; \
    fi

ENV DISPLAY=:99
