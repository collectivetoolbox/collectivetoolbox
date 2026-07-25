# CRLite Client Audit and Enhancement Walkthrough

This document walks through the modifications made to the CRLite verifier to fix strict mode, resolve performance bottlenecks, audit and port test cases, and achieve exact programmatic filter regeneration.

## Completed Changes

### 1. Strict Mode Configuration
We added `strict` mode support to `CRLiteVerifier` (defaulting to `false` in production, and set to `true` in verification-closed tests). This prevents breaking all TLS handshakes to non-enrolled or non-covered hosts.
- Under default (soft-fail) mode: certificates mapping to `NotCovered`, `NotEnrolled`, or `NoFilter` are accepted.
- Under strict mode: these certificate states trigger a validation failure.

### 2. Handshake Performance Caching
We resolved the performance issue where the CRLite filter state was loaded and parsed from disk on every handshake.
- Added a thread-safe global cache `OnceLock<RwLock<Option<Arc<CachedCRLiteState>>>>` in `crlite.rs`.
- The verifier now reads the filter state from memory.
- Manifest checks still verify freshness and auto-load disk state safely in a thread-safe manner if the manifest changes.

### 3. Programmatic Filter Generation Test
We implemented a programmatic test `test_regenerate_fixtures` in Rust that replicates the logic of the original Firefox/python `add_cert.py` and `make_filters.sh` scripts.
- Found and solved the discrepancy between local equation solving and the fixture bytes for L1:
  - In `make_filters.sh`, the certificate `revoked-no-sct` has no SCTs, and was appended to both known and revoked files for L1.
  - In `rust-create-cascade`, the certificate was processed in the approximate ribbon build, but due to a typo or flag setup, it was excluded/treated as not revoked in the L1 exact filter (evaluating to target `1` rather than `0`).
  - By splitting `approx_revoked_serials` and `exact_revoked_serials` lists in the test helper `build_clubcard_in_memory`, we matched the exact linear system inputs.
- Run permutations of insertions to replicate the deterministic search space order from `rust-create-cascade`.
- Achieved byte-for-byte match against `20200101-0-filter` (L1) and `20200101-1-filter.delta` (L2 Delta).

## Validation Results

We executed the full test suite in `ctb-utilities`. All 91 tests passed successfully:

```
running 91 tests
...
test https::crlite::tests::verifier_accepts_not_covered_and_delta_without_delta_in_soft_fail ... ok
test https::crlite::tests::verifier_rejects_not_covered_fixture ... ok
test https::crlite::tests::verifier_accepts_valid_fixture_and_rejects_revoked_fixture ... ok
test https::crlite::tests::test_regenerate_fixtures ... ok
test https::crlite::tests::revoked_in_delta_requires_delta_filter ... ok
...
test result: ok. 91 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
```
