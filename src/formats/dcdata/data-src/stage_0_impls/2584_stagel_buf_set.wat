;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2584 ("StageL buffer set").
;;
;; Overwrites `$buf[$idx]` with `$val` in O(1) time; traps if `$buf` is frozen
;; or `$idx >= len($buf)`.
(module
  ;; 1-byte elements: `u8` (`f203`) and `b` (`f1424`).
  ;; @context(2577 f203)
  ;; @context(2577 f1424)
  (func $buf_set_1b (param $buf i32) (param $idx i64) (param $val i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $buf)))
      (unreachable))
    (i32.store8 (i32.add (i32.load $buf) (i32.wrap_i64 $idx)) $val))

  ;; 4-byte elements: `i32` (`f208`), `u32` (`f207`), and nested lists `l/*` (`2576 275`).
  ;; @context(2577 f208)
  ;; @context(2577 f207)
  ;; @context(2577 2576 275)
  (func $buf_set_4b (param $buf i32) (param $idx i64) (param $val i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $buf)))
      (unreachable))
    (i32.store (i32.add (i32.load $buf) (i32.shl (i32.wrap_i64 $idx) 2)) $val))

  ;; 8-byte elements: `i64` (`f210`) and `u64` (`f209`).
  ;; @context(2577 f210)
  ;; @context(2577 f209)
  (func $buf_set_8b (param $buf i32) (param $idx i64) (param $val i64)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $buf)))
      (unreachable))
    (i64.store (i32.add (i32.load $buf) (i32.shl (i32.wrap_i64 $idx) 3)) $val))
)
