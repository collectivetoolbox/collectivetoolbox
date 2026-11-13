;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2519 ("Wrapping subtract operation").
;;
;; Modular two's-complement subtraction wrapping modulo 2^width without trapping.
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $wrap_sub_u8 (param $a i32) (param $b i32) (result i32)
    (i32.and (i32.sub $a $b) 255))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $wrap_sub_32 (param $a i32) (param $b i32) (result i32)
    (i32.sub $a $b))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $wrap_sub_64 (param $a i64) (param $b i64) (result i64)
    (i64.sub $a $b))
)
