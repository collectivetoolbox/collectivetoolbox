;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2523 ("Bitwise XOR operation").
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $bit_xor_u8 (param $a i32) (param $b i32) (result i32)
    (i32.and (i32.xor $a $b) 255))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $bit_xor_32 (param $a i32) (param $b i32) (result i32)
    (i32.xor $a $b))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $bit_xor_64 (param $a i64) (param $b i64) (result i64)
    (i64.xor $a $b))
)
