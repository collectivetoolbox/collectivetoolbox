;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2544 ("Get element operation").
;;
;; Bounds-checked zero-based indexed element read on `l/T` (`Dc 2576`) and
;; `buf/T` (`Dc 2577`); traps when `$idx >= len`.
(module
  ;; 1-byte elements: `u8` (`f203`) and `b` (`f1424`).
  ;; @context(2576 f203)
  ;; @context(2576 f1424)
  ;; @context(2577 f203)
  ;; @context(2577 f1424)
  (func $seq_get_1b (param $seq i32) (param $idx i64) (result i32)
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $seq)))
      (unreachable))
    (i32.load8_u (i32.add (i32.load $seq) (i32.wrap_i64 $idx))))

  ;; 4-byte elements: `i32` (`f208`), `u32` (`f207`), and nested lists `l/*` (`2576 275`).
  ;; @context(2576 f208)
  ;; @context(2576 f207)
  ;; @context(2576 2576 275)
  ;; @context(2577 f208)
  ;; @context(2577 f207)
  ;; @context(2577 2576 275)
  (func $seq_get_4b (param $seq i32) (param $idx i64) (result i32)
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $seq)))
      (unreachable))
    (i32.load (i32.add (i32.load $seq) (i32.shl (i32.wrap_i64 $idx) 2))))

  ;; 8-byte elements: `i64` (`f210`) and `u64` (`f209`).
  ;; @context(2576 f210)
  ;; @context(2576 f209)
  ;; @context(2577 f210)
  ;; @context(2577 f209)
  (func $seq_get_8b (param $seq i32) (param $idx i64) (result i64)
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $seq)))
      (unreachable))
    (i64.load (i32.add (i32.load $seq) (i32.shl (i32.wrap_i64 $idx) 3))))
)
