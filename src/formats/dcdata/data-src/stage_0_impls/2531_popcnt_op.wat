;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2531 ("Population count operation").
;;
;; Counts set one-bits in a fixed-width integer and returns a u32 (`i32`).
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $popcnt_u8 (param $a i32) (result i32)
    (i32.popcnt (i32.and $a 255)))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $popcnt_32 (param $a i32) (result i32)
    (i32.popcnt $a))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $popcnt_64 (param $a i64) (result i32)
    (i32.wrap_i64 (i64.popcnt $a)))
)
