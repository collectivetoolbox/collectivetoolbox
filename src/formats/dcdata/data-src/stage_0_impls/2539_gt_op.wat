;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2539 ("Greater than comparison").
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $gt_u8 (param $a i32) (param $b i32) (result i32)
    (i32.gt_u (i32.and $a 255) (i32.and $b 255)))

  ;; u32 (`f207`)
  ;; @context(f207)
  (func $gt_u32 (param $a i32) (param $b i32) (result i32)
    (i32.gt_u $a $b))

  ;; i32 (`f208`)
  ;; @context(f208)
  (func $gt_i32 (param $a i32) (param $b i32) (result i32)
    (i32.gt_s $a $b))

  ;; u64 (`f209`)
  ;; @context(f209)
  (func $gt_u64 (param $a i64) (param $b i64) (result i32)
    (i64.gt_u $a $b))

  ;; i64 (`f210`)
  ;; @context(f210)
  (func $gt_i64 (param $a i64) (param $b i64) (result i32)
    (i64.gt_s $a $b))
)
