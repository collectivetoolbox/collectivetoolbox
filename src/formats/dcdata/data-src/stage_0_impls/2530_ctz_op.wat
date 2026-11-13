;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2530 ("Count trailing zeros operation").
;;
;; Counts trailing zero bits in a fixed-width integer and returns a u32 (`i32`).
(module
  ;; u8 (`f203`): set bit 8 sentinel so `0u8` yields 8.
  ;; @context(f203)
  (func $ctz_u8 (param $a i32) (result i32)
    (i32.ctz (i32.or (i32.and $a 255) 256)))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $ctz_32 (param $a i32) (result i32)
    (i32.ctz $a))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $ctz_64 (param $a i64) (result i32)
    (i32.wrap_i64 (i64.ctz $a)))
)
