
# 16 October 2026 Review

**Review Limit**
This is a **partial source review**, not a completed cumulative-diff review: terminal/Git execution is unavailable, so I could not inspect `5aafa2bf3762da8a6d719145cb71f362851c1c6e..HEAD` or run tests. The findings below are present in current source; their introduction within that range remains unverified.

**Findings**
1. **[P1] File-backed binary decoding discards checksums.** The scanner `navigator.rs:2330`, and subsequent materialization reads bytes without verifying it. Corrupted standalone file-backed payloads can therefore decode successfully where in-memory validation rejects them. Journal section hashing provides separate protection, but does not fix the general navigator.

2. **[P1] Forged metadata lengths can panic or exhaust memory.** The file scanner `navigator.rs:2322` before checking whether it fits within the source. A tiny file declaring `usize::MAX` metadata can trigger allocation failure instead of returning a parsing error.

3. **[P1] Strict decoding and strict validation enforce different grammars.** Typed decoding `dc_mixed_serde.rs:150`. Container construction omits mandatory grammar checks: `310 311` decodes successfully as an empty `Vec<u32>`, although the `el.csv:42`. Standalone strict validation checks that header. This leaves the original parser-unification goal incomplete.

4. **[P1] Nested projection can consume unrelated document roots.** `dc_mixed_serde.rs:1374`, `dc_mixed_serde.rs:2260`, and `lib.rs:1951` decoders fall back to global arena roots. They can also mark all roots consumed, suppressing trailing-content errors. For example, a malformed element of `Vec<(DcRawValue, DcRawValue)>` can capture the enclosing list and an unrelated trailing literal as its pair.

5. **[P2] Unsupported date formats silently become epoch nanoseconds.** `dc_mixed_serde.rs:2131` special-cases seconds, then interprets every other format as nanoseconds. The database also declares Panorama date/time formats. Those require proper conversion or an unsupported-format error, not guessed units.

**Completion Assessment**
The refactor remains **Partially Implemented** against the original prompt and subsequent feedback.

The navigator, arena, node-based projection, binary resolver, and restored metadata derives represent substantial consolidation. Several historical findings are already fixed, including tuple framing, pre-epoch encoding, and session-header retry suppression.

Remaining architectural gaps:
- `lib.rs:3198`, including neighboring atoms and optional/equivalent wrappers, contrary to the requested parser/projection boundary.
- Syntax tables are `resolver.rs:665`, not the planned generated static table.
- Configurable arena depth does not govern the standalone validator’s fixed limit.
- Test/status claims need reconciliation. One `challenge_m2_projection.rs:296` and uses tags whose database syntax does not describe numeric arrays.

**Remediation Plan**
1. Establish the exact cumulative diff and fixture integrity before attributing regressions.
2. Assign one subagent to bounded binary scanning/checksum verification, then another to unified syntax ownership. Serialize their navigator edits.
3. Assign projection-scope and temporal-codec fixes after those interfaces stabilize. Require focused failing regressions before fixes.
4. Run affected-crate tests and unsuppressed lints, then independently review the combined changes and update explicit remaining-gap documentation.

The detailed plan and agent boundaries are saved in session memory. No source changes were made in plan mode.