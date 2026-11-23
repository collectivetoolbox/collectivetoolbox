| Script | Packages / Tools to Add | Reason |
| --- | --- | --- |
| `scripts/apt-install-dev-tools` | `hyperfine`, `linux-perf`, `valgrind` | Fast native package installs; `valgrind` provides unprivileged CPU/heap profiling; `hyperfine` gives statistical CLI benchmarking; `linux-perf` gives PMU access when permitted. |
| `scripts/rustup` | `cargo-show-asm`, `flamegraph`, `iai-callgrind-runner` | Cargo CLI tools installed via `cargo install --locked` (matching existing `cargo-deny` pattern). |
| Workspace Crates (as needed) | `dhat`, `iai-callgrind`, `divan` | Dev-dependencies for specific crate benchmarks and heap-allocation tests. |