;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2526 ("Shift right operation").
;;
;; Zero-filling logical right shift on unsigned types (`u8`, `u32`, `u64`) and
;; sign-extending arithmetic right shift on signed types (`i32`, `i64`); traps
;; if the u32 shift count `$cnt >= bit_width(T)`.
(module
  ;; u8 (`f203`): width = 8, logical.
  ;; @context(f203)
  (func $shr_u8 (param $a i32) (param $cnt i32) (result i32)
    (when (i32.ge_u $cnt 8) (unreachable))
    (i32.shr_u (i32.and $a 255) $cnt))

  ;; u32 (`f207`): width = 32, logical.
  ;; @context(f207)
  (func $shr_u32 (param $a i32) (param $cnt i32) (result i32)
    (when (i32.ge_u $cnt 32) (unreachable))
    (i32.shr_u $a $cnt))

  ;; i32 (`f208`): width = 32, arithmetic.
  ;; @context(f208)
  (func $shr_i32 (param $a i32) (param $cnt i32) (result i32)
    (when (i32.ge_u $cnt 32) (unreachable))
    (i32.shr_s $a $cnt))

  ;; u64 (`f209`): width = 64, logical (`$cnt` is u32).
  ;; @context(f209)
  (func $shr_u64 (param $a i64) (param $cnt i32) (result i64)
    (when (i32.ge_u $cnt 64) (unreachable))
    (i64.shr_u $a (i64.extend_i32_u $cnt)))

  ;; i64 (`f210`): width = 64, arithmetic (`$cnt` is u32).
  ;; @context(f210)
  (func $shr_i64 (param $a i64) (param $cnt i32) (result i64)
    (when (i32.ge_u $cnt 64) (unreachable))
    (i64.shr_s $a (i64.extend_i32_u $cnt)))
)
