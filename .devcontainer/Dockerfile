FROM mcr.microsoft.com/devcontainers/base:ubuntu

RUN apt-get update && apt-get install -y --no-install-recommends \
    acl \
    binutils \
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
    xz-utils \
    && npm install -g typescript playwright \
    && npx playwright install-deps chromium \
    && npx playwright install chromium \
    && apt-get clean

# Ensure system-wide umask is 0000 so files created inside container are readable/writable by host users
RUN echo "umask 0000" >> /etc/bash.bashrc \
    && echo "umask 0000" >> /etc/profile \
    && echo "umask 0000" > /etc/profile.d/umask.sh \
    && chmod +x /etc/profile.d/umask.sh \
    && sed -i 's/UMASK\s*022/UMASK 0000/' /etc/login.defs

ENV PLAYWRIGHT_CHROMIUM_ARGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage --disable-gpu"
ENV CHROMIUM_FLAGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage --disable-gpu"
ENV BASH_ENV="/etc/profile.d/umask.sh"

ENV PATH=/root/.cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.92
RUN rustup default 1.92

# Ensure cargo-deny uses sparse protocol for crates.io to check for yanked crates
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN echo "Adding musl target..."
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup target add i586-unknown-linux-musl
RUN rustup target add i586-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown

RUN echo "Installing cargo-deny..."
RUN cargo install --locked cargo-deny

RUN echo "Installing GNU Guix..."
RUN set -eux; \
    curl -fsSL https://codeberg.org/guix/guix/raw/branch/master/etc/guix-install.sh -o /tmp/guix-install.sh; \
    chmod +x /tmp/guix-install.sh; \
    yes | /tmp/guix-install.sh; \
    rm -f /tmp/guix-install.sh; \
    mkdir -p /etc/guix; \
    if [ -d /root/.config/guix/current/share/guix ]; then \
      for key in /root/.config/guix/current/share/guix/*.pub; do \
        [ -e "$key" ] || break; \
        /root/.config/guix/current/bin/guix archive --authorize < "$key"; \
      done; \
    fi

ENV PATH="/root/.config/guix/current/bin:/var/guix/profiles/per-user/root/current-profile/bin:${PATH}"
ENV HOME=/root
WORKDIR /workspaces/ctoolbox

