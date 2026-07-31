FROM mcr.microsoft.com/devcontainers/base:ubuntu

ENV HOME=/root
ENV PATH=/root/.cargo/bin:/root/.config/guix/current/bin:/var/guix/profiles/per-user/root/current-profile/bin:${PATH}
ENV PLAYWRIGHT_CHROMIUM_ARGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage --disable-gpu"
ENV CHROMIUM_FLAGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage --disable-gpu"
ENV BASH_ENV="/etc/profile.d/umask.sh"
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

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
        musl-tools \
        nodejs \
        npm \
        patch \
        perl \
        pkg-config \
        tar \
        tmux \
        xz-utils \
        xxd; \
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

# `sed` is to work around a 502 error. Likely transient; try commenting that out next rebuild.
RUN set -eux; \
    curl -fsSL https://codeberg.org/guix/guix/raw/branch/master/etc/guix-install.sh -o /tmp/guix-install.sh; \
    chmod +x /tmp/guix-install.sh; \
    sed -i 's|https://ftpmirror.gnu.org/gnu/guix/|https://ftp.gnu.org/gnu/guix/|g' /tmp/guix-install.sh; \
    yes | /tmp/guix-install.sh; \
    rm -f /tmp/guix-install.sh; \
    mkdir -p /etc/guix; \
    if [ -d /root/.config/guix/current/share/guix ]; then \
        for key in /root/.config/guix/current/share/guix/*.pub; do \
            [ -e "$key" ] || break; \
            /root/.config/guix/current/bin/guix archive --authorize < "$key"; \
        done; \
    fi

# Pre-build Guix i686 system image tarball in Docker image to avoid re-building on every CI run
COPY scripts/guix/v86-os.scm /tmp/v86-os.scm
COPY src/nopersonality/nopersonality.rs /tmp/nopersonality.rs

RUN set -eux; \
    rustc --edition 2024 --crate-type cdylib -O /tmp/nopersonality.rs -o /tmp/libctb_nopersonality.so; \
    export LD_PRELOAD=/tmp/libctb_nopersonality.so; \
    /root/.config/guix/current/bin/guix-daemon --disable-chroot --build-users-group=guixbuild > /tmp/guix-daemon.log 2>&1 & \
    DAEMON_PID=$!; \
    sleep 2; \
    mkdir -p /var/guix; \
    TARBALL_PATH=$(/root/.config/guix/current/bin/guix system image --system=i686-linux --image-type=tarball /tmp/v86-os.scm); \
    cp "$TARBALL_PATH" /var/guix/v86-system-image.tar.gz; \
    kill "$DAEMON_PID" || true; \
    rm -rf /tmp/v86-os.scm /tmp/nopersonality.rs /tmp/libctb_nopersonality.so /tmp/guix-daemon.log

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

WORKDIR /workspaces/ctoolbox

LABEL org.opencontainers.image.source="https://github.com/collectivetoolbox/collectivetoolbox"
LABEL org.opencontainers.image.description="Collective Toolbox CI and development image"
