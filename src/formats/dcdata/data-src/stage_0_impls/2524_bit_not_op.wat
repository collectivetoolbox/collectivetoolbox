;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2524 ("Bitwise NOT operation").
(module
  ;; u8 (`f203`): invert low 8 bits and keep upper 24 bits clear.
  ;; @context(f203)
  (func $bit_not_u8 (param $a i32) (result i32)
    (i32.xor (i32.and $a 255) 255))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $bit_not_32 (param $a i32) (result i32)
    (i32.xor $a -1))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $bit_not_64 (param $a i64) (result i64)
    (i64.xor $a -1))
)
