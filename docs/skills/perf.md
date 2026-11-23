You have various tools available for working on performance improvement tasks.

| Script | Packages | Reason |
| --- | --- | --- |
| `scripts/apt-install-dev-tools` | `hyperfine`, `linux-perf`, `valgrind` | `valgrind` provides unprivileged CPU/heap profiling; `hyperfine` gives statistical CLI benchmarking; `linux-perf` gives PMU access when permitted. |
| `scripts/rustup` | `cargo-show-asm`, `flamegraph`, `iai-callgrind-runner` | Cargo CLI tools installed via `cargo install --locked` (matching existing `cargo-deny` pattern). |
| Workspace Crates (as needed) | `dhat`, `iai-callgrind`, `divan` | Dev-dependencies for specific crate benchmarks and heap-allocation tests. |

  * **Two Ways We Can Use Dhat**:
    1. **Zero-code profiling**: `valgrind --tool=dhat` (included with `valgrind`) runs against any existing binary with no source changes.
    2. **In-process Rust crate**: Adding `dhat = "0.3"` as an optional/dev dependency enables programmatic heap profiling and unit-test assertions (e.g. asserting that critical parsing loops perform zero unexpected heap allocations). Outputs `dhat-heap.json` viewable in the DHAT browser viewer.
