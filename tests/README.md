These tests currently rely on an external browser as a dependency. They are not required to build or use ctoolbox. I'd like to eventually get them to be runnable using a browser within Guix to minimize binary dependencies for all testing, but haven't looked into that yet.

## Running Browser Tests

To run the Playwright browser test suite against a compiled `ctoolbox` binary:

1. Build `ctoolbox` (e.g. `./build linux-x64` or `./ci`).
2. Run the browser verification script:
   ```bash
   ./scripts/ci-browser-tests
   ```
   Or run CI with browser verification enabled:
   ```bash
   ./ci --browser-tests
   ```

## Windows Shim And Doctest Checks

The shim regression executable tests COM allocation fallbacks, CopyFile2
parameter rejection, native-export preference, and stack bounds including a
failed VirtualQuery. Build it for the supported x64 GNU Windows target:

```bash
mkdir -p target/windows-standalone-tests/windows_shim
x86_64-w64-mingw32-gcc -Wall -Wextra -Wno-attributes -O2 \
   src/build_support/windows/combase_shim_tests.c \
   -o target/windows-standalone-tests/windows_shim/shim.exe -lole32 -lws2_32
WINEDEBUG=-all wine target/windows-standalone-tests/windows_shim/shim.exe
./scripts/win-vm test-suite --debug --no-build --crate ctb_workspace_ipc \
   --standalone-tests-dir target/windows-standalone-tests
```

Rustdoc executables persisted with `--persist-doctests` can be included through
the same standalone-test directory. Compilation with `--no-run` alone is not
native runtime verification. Each executable needs its own uniquely named
parent directory. Standalone programs receive no libtest arguments; unit
binaries run with `--include-ignored`.

The VM serial-protocol regressions do not require a running VM:

```bash
python3 -m unittest discover -s tests -p test_vm_suite.py
```
