;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2528 ("Rotate right operation").
;;
;; Unsigned bitwise right rotation modulo `bit_width(T)` (`u8`, `u32`, `u64`).
(module
  ;; u8 (`f203`): 8-bit rotation modulo 8.
  ;; @context(f203)
  (func $rotr_u8 (param $a i32) (param $cnt i32) (result i32)
    (local $x i32)
    (local $c i32)
    (set $x (i32.and $a 255))
    (set $c (i32.and $cnt 7))
    (i32.and
      (i32.or
        (i32.shr_u $x $c)
        (i32.shl $x (i32.and (i32.sub 8 $c) 7)))
      255))

  ;; u32 (`f207`)
  ;; @context(f207)
  (func $rotr_u32 (param $a i32) (param $cnt i32) (result i32)
    (i32.rotr $a $cnt))

  ;; u64 (`f209`) (`$cnt` is u32)
  ;; @context(f209)
  (func $rotr_u64 (param $a i64) (param $cnt i32) (result i64)
    (i64.rotr $a (i64.extend_i32_u $cnt)))
)
