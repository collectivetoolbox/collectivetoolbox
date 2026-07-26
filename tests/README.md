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
