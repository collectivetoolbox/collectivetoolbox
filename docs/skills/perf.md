# Performance Profiling & Benchmarking Guide

This guide outlines the performance toolchain available in the Collective Toolbox dev environment, when to use each tool, and copy-pasteable recipes for common profiling workflows.

---

## Golden Rules for Performance Work

1. **Verify Correctness First**: Always run the relevant test suite before and after making performance changes. A speedup that breaks correctness is a regression.
2. **Measure, Don't Guess**: Collect concrete profiles (instruction counts, cache misses, allocation counts, or wall-clock percentiles) before optimizing, and verify the delta afterward.
3. **Container Privileges (`perf` vs `valgrind`)**:
   * In standard Docker dev containers, `kernel.perf_event_paranoid` is typically `3` and `CAP_PERFMON` / `CAP_SYS_ADMIN` is not granted; calling `perf_event_open` will return `EPERM`.
   * **In unprivileged containers**: Use **`valgrind` / `callgrind`** (deterministic CPU instruction counts in user space) and **`dhat`** (heap allocations).
   * **On host or privileged containers** (with `--cap-add PERFMON`): Use **`perf`** and **`flamegraph`**.

---

## Toolchain Overview

| Category | Tool | Location / Install | Key Strengths |
| :--- | :--- | :--- | :--- |
| **CLI Benchmarking** | **`hyperfine`** | [scripts/apt-install-dev-tools](file:///workspaces/ctoolbox/scripts/apt-install-dev-tools) | Multi-run warmup, statistical percentiles, parameter scanning, JSON/CSV/Markdown output. |
| **Deterministic CPU & Cache** | **`valgrind` (`callgrind`)** | [scripts/apt-install-dev-tools](file:///workspaces/ctoolbox/scripts/apt-install-dev-tools) | Zero kernel capabilities required; exact instruction counts and cache simulation; line-by-line annotation via `callgrind_annotate`. |
| **Hardware Counters & Sampling** | **`linux-perf` (`perf`)** | [scripts/apt-install-dev-tools](file:///workspaces/ctoolbox/scripts/apt-install-dev-tools) | PMU hardware counters (cycles, instructions, branch & cache misses); statistical sampling (`perf record`/`report`/`annotate`). |
| **Call-Stack Visualization** | **`flamegraph`** | [scripts/rustup](file:///workspaces/ctoolbox/scripts/rustup) (`cargo flamegraph`) | Generates interactive SVG flame graphs of CPU hot spots. |
| **Codegen & Assembly** | **`cargo-show-asm`** | [scripts/rustup](file:///workspaces/ctoolbox/scripts/rustup) (`cargo asm`) | Inspects generated assembly, LLVM-IR, MIR, or WASM side-by-side with Rust source. |
| **Heap & Allocation Profiling** | **`dhat`** | APT (`valgrind --tool=dhat`) or Rust crate `dhat` | Lifetime allocations, peak heap usage, short-lived churn, and zero-allocation assertions. |
| **In-Process Microbenchmarks** | **`iai-callgrind`** | Runner in [scripts/rustup](file:///workspaces/ctoolbox/scripts/rustup); crate `0.16.1` in `Cargo.toml` | Deterministic instruction-count benchmarks immune to CPU throttling and CI jitter. |
| **In-Process Microbenchmarks** | **`divan`** | Rust crate in `Cargo.toml` | Fast, ergonomic wall-clock microbenchmarks with throughput tracking (`#[divan::bench]`). |

---

## Workflows & Recipes

### 1. End-to-End CLI Benchmarking with `hyperfine`

Use `hyperfine` to measure CLI command execution time with warmup runs and statistical rigor:

```bash
# Basic benchmark with 3 warmup runs
hyperfine --warmup 3 './target/x86_64-unknown-linux-musl/release/ctoolbox <subcommand> <args>'

# Compare baseline binary vs optimized binary and export markdown table
hyperfine --warmup 3 \
  -n "baseline" './target-baseline/release/ctoolbox run input.dc' \
  -n "optimized" './target/x86_64-unknown-linux-musl/release/ctoolbox run input.dc' \
  --export-markdown /tmp/bench-results.md

# Parameterized benchmark across thread counts or chunk sizes
hyperfine --parameter-scan threads 1 8 './target/release/ctoolbox process --threads {threads} data.bin'
```

---

### 2. CPU Profiling & Hotspots

#### Option A: Container-Safe Profiling with `valgrind` / `callgrind`
Because `callgrind` runs in user-space dynamic binary translation, it does not require Linux PMU capabilities and works reliably inside unprivileged containers:

```bash
# 1. Run binary under Callgrind
valgrind --tool=callgrind --callgrind-out-file=/tmp/callgrind.out \
  ./target/x86_64-unknown-linux-musl/release/ctoolbox <args>

# 2. View hot functions annotated by source line
callgrind_annotate --auto=yes /tmp/callgrind.out | head -n 60

# 3. Focus on a specific source file
callgrind_annotate --include=/workspaces/ctoolbox/src /tmp/callgrind.out | head -n 60
```

#### Option B: Hardware Counters & Statistical Sampling with `perf` (Privileged / Host)
When running on the host or inside a container started with `--cap-add PERFMON`:

```bash
# Hardware counters: cycles, instructions, branch misses, cache misses
perf stat ./target/x86_64-unknown-linux-musl/release/ctoolbox <args>

# Sampling profile with call stacks
perf record -g -- ./target/x86_64-unknown-linux-musl/release/ctoolbox <args>

# Interactive report
perf report

# Disassembly with instruction hotspot percentage
perf annotate <function_name>
```

#### Option C: Flame Graphs with `cargo flamegraph`
```bash
# Generate flamegraph.svg for a release binary
cargo flamegraph --bin ctoolbox -- <args>
```

---

### 3. Codegen & Assembly Inspection with `cargo asm`

Use `cargo asm` to inspect inlining, auto-vectorization (SIMD), bounds check elimination, and compiler optimizations:

```bash
# 1. List available symbols in a workspace crate
cargo asm -p ctb-runtime --lib

# 2. Show assembly for a function with interleaved Rust source
cargo asm -p ctb-runtime --lib "ctb_runtime::cli::execute_cli_run" --rust

# 3. View LLVM-IR or MIR instead of assembly
cargo asm -p ctb-runtime --lib "my_function" --llvm
cargo asm -p ctb-runtime --lib "my_function" --mir

# 4. View WebAssembly output for wasm targets
cargo asm -p ctb-runtime --target wasm32-unknown-unknown --lib "my_function"
```

---

### 4. Memory & Allocation Profiling with `dhat`

#### Approach 1: Zero-Code Profiling via Valgrind
Profile any pre-built binary without modifying source code:

```bash
valgrind --tool=dhat --dhat-out-file=/tmp/dhat.json \
  ./target/x86_64-unknown-linux-musl/release/ctoolbox <args>
```
Load `/tmp/dhat.json` into the browser viewer at [https://nnethercote.github.io/dh_view/viewer.html](https://nnethercote.github.io/dh_view/viewer.html) to see allocation backtraces, peak heap memory, and short-lived block churn.

#### Approach 2: Programmatic In-Process Profiling & Test Assertions
Add `dhat` to `[dev-dependencies]` in the target crate:

```toml
[dev-dependencies]
dhat = "0.3"
```

In integration tests or benchmarks, enforce zero-allocation invariants:

```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn hot_path_does_not_allocate() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // Run hot loop
    hot_path_function();

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0, "Hot path performed unexpected heap allocations!");
}
```

---

### 5. In-Process Benchmarks

#### Deterministic Benchmarks (`iai-callgrind`)
For regression detection in CI that is immune to CPU throttling or virtualized scheduling jitter:
1. Ensure `valgrind` is installed (`scripts/apt-install-dev-tools`) and runner is installed (`scripts/rustup`).
2. Add to crate `Cargo.toml`:
   ```toml
   [dev-dependencies]
   iai-callgrind = "=0.16.1"

   [[bench]]
   name = "my_benchmark"
   harness = false
   ```
3. Run benchmarks:
   ```bash
   cargo bench -p <crate> --bench my_benchmark
   ```

#### Ergonomic Wall-Clock Benchmarks (`divan`)
For fast local experimentation with throughput tracking:
```toml
[dev-dependencies]
divan = "0.1"

[[bench]]
name = "my_benchmark"
harness = false
```
```rust
fn main() {
    divan::main();
}

#[divan::bench(args = [1024, 4096, 16384])]
fn bench_throughput(bencher: divan::Bencher, size: usize) {
    let data = vec![0u8; size];
    bencher
        .counter(divan::counter::BytesCount::new(size))
        .bench(|| {
            process_buffer(divan::black_box(&data));
        });
}
```
