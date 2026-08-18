# syntax=docker/dockerfile:1
FROM mcr.microsoft.com/devcontainers/base:ubuntu

ENV HOME=/root
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8
ENV GUIX_LOCPATH=/var/guix/profiles/per-user/root/current-profile/lib/locale:/root/.guix-profile/lib/locale
ENV PATH=/root/.cargo/bin:/root/.config/guix/current/bin:/var/guix/profiles/per-user/root/current-profile/bin:${PATH}
ENV PLAYWRIGHT_CHROMIUM_ARGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage --disable-gpu"
ENV CHROMIUM_FLAGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage --disable-gpu"
ENV BASH_ENV="/etc/profile.d/umask.sh"
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV GUILE_AUTO_COMPILE=0
ENV GUILE_WARN_DEPRECATED=no
ENV container=docker

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        acl \
        binutils \
        brotli \
        build-essential \
        ca-certificates \
        clang \
        coreutils \
        curl \
        file \
        git \
        locales \
        musl-tools \
        nodejs \
        npm \
        patch \
        perl \
        pkg-config \
        proot \
        tar \
        xz-utils \
        xxd; \
    locale-gen en_US.UTF-8; \
    npm install -g typescript playwright; \
    npx playwright install-deps chromium; \
    npx playwright install chromium; \
    apt-get clean

# Ensure system-wide umask is 0000 so files created inside container are readable/writable by host users
RUN set -eux; \
    echo "umask 0000" >> /etc/bash.bashrc; \
    echo "umask 0000" >> /etc/profile; \
    echo "umask 0000" > /etc/profile.d/umask.sh; \
    chmod +x /etc/profile.d/umask.sh; \
    sed -i 's/UMASK\s*022/UMASK 0000/' /etc/login.defs

RUN set -eux; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.92; \
    rustup default 1.92; \
    rustup target add x86_64-unknown-linux-musl; \
    rustup target add i586-unknown-linux-musl; \
    rustup target add i586-unknown-linux-gnu; \
    rustup target add wasm32-unknown-unknown; \
    cargo install --locked cargo-deny

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

# `sed` is to work around a 502 error. Likely transient; try commenting that out next rebuild.
RUN set -eux; \
    curl -fsSL https://codeberg.org/guix/guix/raw/branch/master/etc/guix-install.sh -o /tmp/guix-install.sh; \
    chmod +x /tmp/guix-install.sh; \
    sed -i 's|https://ftpmirror.gnu.org/gnu/guix/|https://ftp.gnu.org/gnu/guix/|g' /tmp/guix-install.sh; \
    sed -i 's/sys_maybe_setup_apparmor/true/g' /tmp/guix-install.sh; \
    yes | /tmp/guix-install.sh; \
    rm -f /tmp/guix-install.sh; \
    rm -rf /etc/apparmor.d/guix* /etc/apparmor.d/tunables/guix* 2>/dev/null || true; \
    mkdir -p /etc/guix /var/log/guix/drvs; \
    chown -R root:guixbuild /var/guix /var/log/guix /gnu/store; \
    chmod 1775 /gnu/store /var/guix; \
    chmod -R 1777 /var/log/guix; \
    for key in /var/guix/profiles/per-user/root/current-profile/share/guix/*.pub \
               /root/.config/guix/current/share/guix/*.pub \
               /usr/local/share/guix/*.pub; do \
        if [ -f "$key" ]; then \
            guix archive --authorize < "$key" || true; \
        fi; \
    done; \
    chmod -R u+w /gnu/store; \
    find /gnu/store -name "perform-download.scm" -exec sed -i 's/(when (zero? (getuid))/(when #f/g' {} +; \
    find /gnu/store -name "perform-download.go" -delete

# Pre-build Guix packages and system image in Docker for layer caching.
# Each step is a separate layer so failures don't re-run earlier steps.
COPY scripts/guix /tmp/scripts/guix
COPY src/nopersonality /tmp/src/nopersonality

# Layer 1: Native i686 Dillo build (smoke test for Guix sandboxing)
# RUN --security=insecure set -eux; \
#     /tmp/scripts/guix/build-v86-guix-image.sh --build-dillo-native || { \
#         st=$?; \
#         echo "=== [RUN layer 1 failure diagnostic: exit code $st] ===" >&2; \
#         if [ -f /proc/meminfo ]; then cat /proc/meminfo | head -n 15 >&2 || true; fi; \
#         if [ -f /proc/vmstat ]; then grep -i oom /proc/vmstat >&2 || true; fi; \
#         if [ -f /sys/fs/cgroup/memory.events ]; then cat /sys/fs/cgroup/memory.events >&2 || true; fi; \
#         if [ -f /sys/fs/cgroup/memory/memory.oom_control ]; then cat /sys/fs/cgroup/memory/memory.oom_control >&2 || true; fi; \
#         if command -v dmesg >/dev/null 2>&1; then dmesg 2>/dev/null | tail -n 50 >&2 || true; fi; \
#         if [ -f /tmp/guix-daemon.log ]; then echo "=== /tmp/guix-daemon.log ===" >&2; tail -n 100 /tmp/guix-daemon.log >&2 || true; fi; \
#         exit "$st"; \
#     }

# Layer 2: Cross-compile Dillo x86_64→i686 (smoke test for cross-toolchain)
RUN --security=insecure set -eux; \
    /tmp/scripts/guix/build-v86-guix-image.sh --cross-dillo || { \
        st=$?; \
        echo "=== [RUN layer 2 failure diagnostic: exit code $st] ===" >&2; \
        if [ -f /proc/meminfo ]; then cat /proc/meminfo | head -n 15 >&2 || true; fi; \
        if [ -f /proc/vmstat ]; then grep -i oom /proc/vmstat >&2 || true; fi; \
        if [ -f /tmp/guix-daemon.log ]; then tail -n 100 /tmp/guix-daemon.log >&2 || true; fi; \
        exit "$st"; \
    }

# Layer 3: Cross-compile GNU Icecat x86_64→i686
RUN --security=insecure set -eux; \
    /tmp/scripts/guix/build-v86-guix-image.sh --cross-icecat || { \
        st=$?; \
        echo "=== [RUN layer 3 failure diagnostic: exit code $st] ===" >&2; \
        if [ -f /proc/meminfo ]; then cat /proc/meminfo | head -n 15 >&2 || true; fi; \
        if [ -f /proc/vmstat ]; then grep -i oom /proc/vmstat >&2 || true; fi; \
        if [ -f /tmp/guix-daemon.log ]; then tail -n 100 /tmp/guix-daemon.log >&2 || true; fi; \
        exit "$st"; \
    }

# Layer 4: Build Guix i686 system image tarball
RUN --security=insecure set -eux; \
    /tmp/scripts/guix/build-v86-guix-image.sh --prebuild-tarball /var/guix/v86-system-image.tar.gz --disable-cross || { \
        st=$?; \
        echo "=== [RUN layer 4 failure diagnostic: exit code $st] ===" >&2; \
        if [ -f /proc/meminfo ]; then cat /proc/meminfo | head -n 15 >&2 || true; fi; \
        if [ -f /proc/vmstat ]; then grep -i oom /proc/vmstat >&2 || true; fi; \
        if [ -f /tmp/guix-daemon.log ]; then tail -n 100 /tmp/guix-daemon.log >&2 || true; fi; \
        exit "$st"; \
    }; \
    rm -rf /tmp/scripts /tmp/src

# Pre-build TypeScript compiler tarball in Docker image to avoid re-building on every CI run
#ENV NOCACHEJULYTHIRTY=2
COPY Cargo.toml Cargo.lock /tmp/ctb-ts-build/
COPY src /tmp/ctb-ts-build/src
COPY vendor /tmp/ctb-ts-build/vendor

# Patch to avoid network access so `cargo run --offline` can work
RUN set -eux; \
    mkdir -p /tmp/ctb-ts-build/.cargo; \
    printf '%s\n' \
        '[source.crates-io]' \
        'replace-with = "vendored-sources"' \
        '' \
        '[source.vendored-sources]' \
        'directory = "vendor/ctb-vendored/vendor"' \
        '' \
        '[source."https://github.com/boa-dev/boa.git"]' \
        'git = "https://github.com/boa-dev/boa.git"' \
        'rev = "ffec9244d4267406d66aef8b3c8a1d89730df5b4"' \
        'replace-with = "vendored-sources"' \
        '' \
        '[source."https://github.com/rust-dark-light/dark-light.git"]' \
        'git = "https://github.com/rust-dark-light/dark-light.git"' \
        'rev = "0f18d2fbcaa5d1c175db8aae7d53428988d7e961"' \
        'replace-with = "vendored-sources"' \
        > /tmp/ctb-ts-build/.cargo/config.toml; \
    find /tmp/ctb-ts-build/vendor -name '*.gitignore-2528979c-1cdc-49a3-83ec-a1cdea2ed741.moved' | while read -r file; do \
        mv "$file" "${file%-2528979c-1cdc-49a3-83ec-a1cdea2ed741.moved}"; \
    done; \
    cd /tmp/ctb-ts-build; \
    cargo run --offline -p ctb-formats-javascript-bootstrap-ts --bin bootstrap_ts --release -- ./vendor/TypeScript; \
    mkdir -p /var/cache/ctoolbox; \
    cp vendor/TypeScript-built.tar /var/cache/ctoolbox/TypeScript-built.tar; \
    cd /; \
    rm -r /tmp/ctb-ts-build

RUN set -eux; \
    dpkg --add-architecture i386; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        tmux \
        # autoconf \
        # automake \
        # bison \
        build-essential \
        gcc-multilib \
        # libexpat1-dev \
        # libexpat1-dev:i386 \
        # libffi-dev \
        # libffi-dev:i386 \
        # libssl-dev \
        # libssl-dev:i386 \
        # libwayland-dev \
        # libwayland-dev:i386 \
        mingw-w64 \
        musl-dev:i386; \
    apt-get clean

WORKDIR /workspaces/ctoolbox

LABEL org.opencontainers.image.source="https://github.com/collectivetoolbox/collectivetoolbox"
LABEL org.opencontainers.image.description="Collective Toolbox CI and development image"
