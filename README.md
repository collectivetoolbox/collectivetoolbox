<img src="assets/logo.svg" alt="Collective Toolbox Logo" width="400"/>

## Collective Toolbox: Overview

Collective Toolbox: A graph‑based workspace for linking documents and data—across local and shared knowledge—so you can query, explore, and build reports with clarity.

Built with a focus on user respect and transparency. Free‑software licensing for study and modification; paid hosting planned for convenience and reliability.

## Disclaimer

Please note: this is currently a hobby project provided "as is" (see the license for the full disclaimer) and is not suited for use for security critical, business critical, or other important purposes.

## Licensing

This is licensed primarily under AGPL-3+. Parts of the code are reused under MIT and other license terms, see individual files for details.

The files in `build_support/bin/seabios_tool.rs` and `build_suppport/seabios_build.rs` are derived from SeaBIOS. Unfortunately, I don't see any "or later" note for the licensing of the scripts they're based on, so it appears that to use the combined ctoolbox source under later AGPL versions, those files would need to be discarded and replaced with new implementations, as I understand it.

The standard ncurses terminfo database is included here under its own historically messy terms; see src/utilities/data/terminfo.

## Dev notes

- A couple of other git repositories are referenced as submodules:
  - ctb-c-vendored lives at vendor/ (required to build)
  - ctb-vendored lives at vendor/ctb-vendored (optional, not needed just to build)

- Installing build dependencies: See the comments in the `build` script
- Checking for unused dependencies: `cd ~/ctoolbox || exit 1; cargo shear`
- Checking dependencies for license issues: `cd ~/ctoolbox || exit 1; cargo deny check`
- Checking what cat dragged in what unwanted dependency: `cd ~/ctoolbox; cargo tree --invert (dependency-name)` or for unwanted features `cd ~/ctoolbox || exit 1; cargo tree -e features --invert (dependency-name)`
- Automatically fix some lints (may need `cargo clippy --workspace --fix --broken-code`): `cd ~/ctoolbox || exit 1; ~/ctoolbox/scripts/format`
- Saving dependencies: `cd ~/ctoolbox/vendor/ctb-vendored || exit 1; ~/ctoolbox/scripts/vendor-dependencies`
- Update Guix package: `cd ~/ctoolbox || exit 1; cp packaging/guix/rust-crates.tmpl packaging/guix/generated/ctb-workspace-rust-crates.scm; guix import --insert=packaging/guix/generated/ctb-workspace-rust-crates.scm crate --lockfile=Cargo.lock ctb-workspace`
- Build with Guix: `cd ~/ctoolbox || exit 1; ./packaging/guix-build`
- Guix debugging: `cd /tmp/guix-build-ctoolbox-0.1.0.drv-0 || exit 1; guix shell --no-grafts -f ~/ctoolbox/packaging/guix/generated/ctoolbox.scm -C -D strace`
- Macro troubleshooting: `pushd ctoolbox || exit 1; cargo expand --lib 'io::webui::controllers::graph' | less; popd` (but this is less useful because "macro hygiene" rules mean what visually looks like the expanded code is not actually how it's interpreted)
- Hot reload may become possible using https://docs.rs/subsecond/0.7.0-alpha.1/subsecond/
- Installing cargo-vet: `cargo install --locked cargo-vet`
- Importing cargo-vet rules: `cargo vet import mozilla`
- Running cargo-vet: `cargo vet` (output looks something like `Vetting Succeeded (79 fully audited, 526 exempted)`)
- Installing cargo-nextest (slower, but allows seeing slow tests): `cargo install cargo-nextest --locked` and then uncomment the `cargo nextest` commands in build script and comment out the `cargo test` ones
- Installing cargo bloat: `cargo install cargo-bloat`
- Seeing largest functions: `cd ~/ctoolbox || exit 1; cargo bloat --release -n 20`
- Seeing largest dependencies: `cd ~/ctoolbox || exit 1; cargo bloat --release --crates -n 150`
- Seeing largest source: `cd ~/ctoolbox/src || exit 1; shopt -s globstar; wc -l **/*`
- Install workspace-lints: `cargo install cargo-workspace-lints`
- Check for workspace lint setup correct: `cd ~/ctoolbox || exit 1; cargo workspace-lints -v`
- Quick build and run: `cd ~/ctoolbox || exit 1; RUST_BACKTRACE=1 ./run-linux`
- Run a single test: `cargo test spans_attach_request_and_stream_fields -- --nocapture`
- Removing an accidental submodule: `git rm -f --cached folder` - it does *not* delete the folder.
- Creating Docker containers:
  - Note: Docker containers are NOT required to build ctoolbox or probably the v86 image, but are how I build the v86 image. I want to minimize dependency on, so the Dockerfiles use a Debian base image without extras to start with and build everything on top of that. CI runs in a container that's already partly built. It should be possible to build the image on a native Debian installation using the same scripts, though it may require some manual wrangling and I haven't tried it.
  - Build v86 image in Docker container (used for building v86 image; this will take several hours; 400GB free disk space recommended though you may be able to get away with less): `pushd ~/ctoolbox/ || exit 1; ./scripts/build-docker-image; notify-send "Docker command finished" || true`
  - Tag v86 builder Docker container (remember to update the image hash and date to match): `docker tag 091481791716 ghcr.io/collectivetoolbox/collectivetoolbox-2026-sept-5-build:latest`
  - Protecting a Docker image from `docker image prune`: `docker create --name keep-ctb-builder-2026-sept-5-build ghcr.io/collectivetoolbox/collectivetoolbox-2026-sept-5-build:latest`
  - Extract the built v86 image from the Docker container to avoid rebuilding on the host (pass the tag of the large builder image): `cd ~/ctoolbox || exit 1; ./scripts/extract-v86-images ghcr.io/collectivetoolbox/collectivetoolbox-2026-sept-5-build:latest`
  - Build small Docker container (NOT required, used for CI and dev container): `pushd ~/ctoolbox/ || exit 1; ./scripts/build-docker-image --use-built; notify-send "Docker command finished" || true`
  - Tag small Docker container (remember to update the image hash and date to match; or pass `-t <tag>` to build-docker-image): `docker tag 091481791716 ghcr.io/collectivetoolbox/collectivetoolbox-2026-sept-5:latest`

- Handlebars `{{ var }}` is escaped; `{{{ var }}}` is unescaped.

- Seeing outcomes from a `crater` run: tar --zstd -xOf all.tar.zst --wildcards 'broken/gh/collectivetoolbox/collectivetoolbox/*/*.txt'

#### Pushing Docker container

##### Step 1: Create the Classic Token

   1. Go to GitHub.com and click your profile picture (top right) -> Settings.
   2. Scroll to the bottom of the left sidebar and click Developer settings.
   3. Expand Personal access tokens and select Tokens (classic).
   4. Click Generate new token -> Generate new token (classic).
   5. Give it a name (e.g., "Docker Push Token").
   6. Check only the box for write:packages. (Note: read:packages will auto-select, which is fine).
   7. Scroll to the bottom, click Generate token, and copy the token immediately (you will never see it again).

##### Step 2: Push Image

Remember to update the date to match.

`cd ~/ctoolbox || exit 1; ./scripts/docker-push 2026-jul-30`

* Press Enter. It will ask for a Password.
* Paste the classic token you copied in Step 1 and press Enter.
* It should say Login Succeeded.

Running in docker: `docker run -it ghcr.io/collectivetoolbox/collectivetoolbox-2026-jul-30-2 bash`

### Updating:

- Updating Rust: Edit `scripts/rustup` to the versions and default that you want, and then run it.
- Updating dependencies: `cd ~/ctoolbox || exit 1; cargo cooldown update --workspace; wget https://github.com/ua-parser/uap-core/raw/refs/heads/master/LICENSE -O src/formats/useragent/data/regexes.yaml.LICENSE || exit 1; wget https://raw.github.com/ua-parser/uap-core/master/regexes.yaml -O src/formats/useragent/data/regexes.yaml || exit 1`
- Updating dependencies past unstable version (requires `cargo cooldown install cargo-edit`): `cd ~/ctoolbox || exit 1; cargo cooldown upgrade -i allow --dry-run` or `cd ~/ctoolbox || exit 1; cargo cooldown upgrade -i allow` to save them
  - Note that this will *not* work if the "edition" is not set to the latest in Cargo.toml, so make sure the edition is up to date.

### Building:

See comments in ./build for build prerequisites. Currently the instructions assume it's being built on Debian 13.

Search for "FOR STATIC BUILD" and "FOR DYNAMIC BUILD" to find sections of Cargo files that need to be changed to build dynamically on Linux. Note that I don't advise using dynamic builds on Linux; they'll likely be missing things or not work as expected, but can be useful for troubleshooting.

- Default (auto-detect host platform): `cd ~/ctoolbox || exit 1; ./build --release`
- List supported targets: `cd ~/ctoolbox || exit 1; ./build --list-platforms`
- Build a specific target:
  - Linux x86-64: `cd ~/ctoolbox || exit 1; ./build linux-x64 --release` or `cd ~/ctoolbox || exit 1; ./build linux-x64 --debug`
  - Linux x86: `cd ~/ctoolbox || exit 1; ./build linux-x86 --release` or `cd ~/ctoolbox || exit 1; ./build linux-x86 --debug`
- Build all supported targets: `cd ~/ctoolbox || exit 1; ./build all --release`

Build outputs land in `built/<target>/` and include:

- `ctoolbox` (the main binary)
- `ctoolbox-installer` (the installer/updater binary)

### Running

See index page at https://collectivetoolbox.com or in `assets/views/index.hbs` for system requirements.

- Running with some extra debug information: `RUST_LOG="warn,ctoolbox=debug,tower_http=debug,hyper=warn,axum::rejection=trace" ctoolbox`
- Running with lots of extra debug information (every HTTP request): `RUST_LOG="debug,ctoolbox=debug,tower_http=debug" ./deploy/ctoolbox`

### Deploying

Server & local prerequisites:

- Server (Debian/Linux): `rsync`, `curl`, `libcap2-bin` (`setcap`), `sudo`, and `systemd` with user lingering (`loginctl enable-linger <server-user>`), plus three separated accounts (deployer, sudoer, and server runner).
- Local machine: `jq`, `rsync`, `openssh-client` (plus `sshpass` only if using password authentication instead of an SSH key, and `7z` optionally for local `../ctb-deploys/` archives).

Target configuration (`~/ctb-deploy.*`, or in a directory passed via `--config-dir <DIR>` / `--target <DIR>`):

- `~/ctb-deploy.username` — deployer SSH username (uploads releases, binaries, and systemd unit)
- `~/ctb-deploy.sudoer-username` — sudoer SSH username (runs `scripts/setup-permissions-groups` and `setcap`)
- `~/ctb-deploy.server-username` — service runner SSH username (runs the `ctoolbox` `systemd --user` service)
- `~/ctb-deploy.ip` — server IP or hostname
- `~/ctb-deploy.port` — server SSH port
- `~/ctb-deploy.key` *(optional, or pass `--ssh-key <PATH>`)* — SSH private key; avoids `sshpass` and SSH password prompts
- `~/ctb-deploy.global-graph-pass` *(optional)* — password for the server's `global` graph user; prompted interactively if omitted
- Upload the server configuration to the server user's storage dir (Linux):
  `~/.local/share/collectivetoolbox/config/pc_settings.json`

Signing keys (developer flow):

- Generate an Ed25519 keypair and write it to your local `pc_settings.json`:
  `./scripts/run-dev-ctb -- ctb-dev-key-create --write`
- Copy the generated *public* key (`dev_signing_public_key`) to the server's `pc_settings.json` as `release_public_key`.

Deployment script (`./deploy` skips lints and tests during compilation, as CI runs those):

- `./deploy [all] --skip-clean` (or `--clean`): default `all` mode bumps the patch version, builds `linux-x64` and `linux-x86` release binaries + docs, archives source & vendored dependencies, updates Guix/Homebrew packaging, signs and chunks releases (`ctb-dev-sign`), uploads releases, deploys the `linux-x64` server binary and `.rsrc` bundles, restarts the `systemd --user` service, verifies release signatures/chunks (`ctb-dev-release-check`) and `/api/update-status`, logs in/initializes the `global` graph user, and tags `v<version>`.
- `./deploy quick --skip-clean` (or `--clean`): runs the same pipeline for `linux-x64` only (skipping `linux-x86` cross-compilation).
- Useful options:
  - `--clean` / `--skip-clean`: required when building unless `--reuse-built`, `--no-build`, or `--target vm` is used.
  - `--reuse-built`: reuse existing `built/<target>/ctoolbox` binaries whose embedded commit matches `HEAD`; rebuild only missing/stale targets.
  - `--no-build`: skip building completely (requires up-to-date binaries matching `HEAD`).
  - `--dry-run` (`--no-git-bump`): skip version bump, packaging file mutations, git commits, and git tags.
  - `--target vm`: deploy to the local QEMU Debian sandbox VM (`127.0.0.1:2224`, implies `--dry-run`; see [docs/testing-deployment.md](docs/testing-deployment.md)).
  - `--mock-artifacts` / `--allow-dirty`: speed up local VM deploy tests.

If SSH key authentication (`ctb-deploy.key` / `--ssh-key`) is not configured, you will be prompted for SSH passwords for the deployer, server, and sudoer accounts; if `ctb-deploy.global-graph-pass` is not present, you will also be prompted for the global graph user password.
To prune old release chunks and manifests on the server later, run `ctoolbox ctb-dev-release-expire --older-than <DAYS>`.

### Platforms

With the caveat that this software is unfinished:

| Platform | Known issues, if any |
|----------|----------|
| Linux | OK |
| Native Windows, Mac, iOS... | ? Probably not simple to build without Win/Mac machines handy |
| Webkit; Blink    | Pausing web version for now as due to lack of normal Rust threading, it will presumably need emulation instead (perhaps blocked by https://github.com/copy/v86/issues/133) |
| Firefox | Broken because of https://bugzil.la/1360870; https://bugzil.la/1320796 |
| PWA (iOS/Android) | No blockers, but unimplemented |

Development note:
- FreeBSD sysroot:
`./scripts/download-freebsd-sysroot.py --manifest vendor/freebsd-sysroot-manifest.json` recreates it.

### Pregenerated and blobs

Not comprehensive. I'm letting auto-generated *library bindings* slide as that seems like a different case.

- ICU4X: https://codeberg.org/guix/guix/issues/2401
- unicode-width (pregenerated)
- webview2-com-sys (blobs)
- winapi (blobs)
