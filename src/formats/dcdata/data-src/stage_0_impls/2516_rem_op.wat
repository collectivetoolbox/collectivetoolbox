;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2516 ("Remainder operation").
;;
;; Checked truncated integer remainder; traps on zero divisor and explicitly
;; traps on signed `MIN % -1` (where raw WASM `rem_s` would otherwise yield 0).
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $rem_u8 (param $a i32) (param $b i32) (result i32)
    (i32.rem_u (i32.and $a 255) (i32.and $b 255)))

  ;; u32 (`f207`)
  ;; @context(f207)
  (func $rem_u32 (param $a i32) (param $b i32) (result i32)
    (i32.rem_u $a $b))

  ;; i32 (`f208`)
  ;; @context(f208)
  (func $rem_i32 (param $a i32) (param $b i32) (result i32)
    (when (i32.and (i32.eq $a -2147483648) (i32.eq $b -1))
      (unreachable))
    (i32.rem_s $a $b))

  ;; u64 (`f209`)
  ;; @context(f209)
  (func $rem_u64 (param $a i64) (param $b i64) (result i64)
    (i64.rem_u $a $b))

  ;; i64 (`f210`)
  ;; @context(f210)
  (func $rem_i64 (param $a i64) (param $b i64) (result i64)
    (when (i32.and (i64.eq $a -9223372036854775808) (i64.eq $b -1))
      (unreachable))
    (i64.rem_s $a $b))
)
