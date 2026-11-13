;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2529 ("Count leading zeros operation").
;;
;; Counts leading zero bits in a fixed-width integer and returns a u32 (`i32`).
(module
  ;; u8 (`f203`): subtract upper 24 zero bits of the 32-bit lane.
  ;; @context(f203)
  (func $clz_u8 (param $a i32) (result i32)
    (i32.sub (i32.clz (i32.and $a 255)) 24))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $clz_32 (param $a i32) (result i32)
    (i32.clz $a))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $clz_64 (param $a i64) (result i32)
    (i32.wrap_i64 (i64.clz $a)))
)
