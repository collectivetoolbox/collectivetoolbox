;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2541 ("Convert value operation").
;;
;; Checked value-preserving numeric conversions across all 25 `(src, dst)` pairs
;; in `{u8: f203, i32: f208, u32: f207, i64: f210, u64: f209}`, indexed via
;; `Dc 302` ("Type conversion"): `@context(302 <src> <dst>)`.
(module
  ;; Lossless u8 widening/identity to 32-bit (`u8 -> u8 | i32 | u32`).
  ;; @context(302 f203 f203)
  ;; @context(302 f203 f208)
  ;; @context(302 f203 f207)
  (func $conv_u8_to_32 (param $x i32) (result i32)
    (i32.and $x 255))

  ;; Lossless u8 widening to 64-bit (`u8 -> i64 | u64`).
  ;; @context(302 f203 f210)
  ;; @context(302 f203 f209)
  (func $conv_u8_to_64 (param $x i32) (result i64)
    (i64.extend_i32_u (i32.and $x 255)))

  ;; 32-bit identity (`i32 -> i32`, `u32 -> u32`).
  ;; @context(302 f208 f208)
  ;; @context(302 f207 f207)
  (func $conv_id_32 (param $x i32) (result i32)
    (get $x))

  ;; Checked 32-bit narrowing to u8 (`i32 -> u8`, `u32 -> u8`): traps outside 0..=255.
  ;; @context(302 f208 f203)
  ;; @context(302 f207 f203)
  (func $conv_32_to_u8 (param $x i32) (result i32)
    (when (i32.gt_u $x 255) (unreachable))
    (get $x))

  ;; Checked 32-bit sign-change (`i32 -> u32`, `u32 -> i32`): traps when sign bit is set.
  ;; @context(302 f208 f207)
  ;; @context(302 f207 f208)
  (func $conv_32_nonneg (param $x i32) (result i32)
    (when (i32.lt_s $x 0) (unreachable))
    (get $x))

  ;; Lossless signed widening `i32 -> i64`.
  ;; @context(302 f208 f210)
  (func $conv_i32_to_i64 (param $x i32) (result i64)
    (i64.extend_i32_s $x))

  ;; Checked widening `i32 -> u64`: traps when negative.
  ;; @context(302 f208 f209)
  (func $conv_i32_to_u64 (param $x i32) (result i64)
    (when (i32.lt_s $x 0) (unreachable))
    (i64.extend_i32_u $x))

  ;; Lossless unsigned widening `u32 -> i64 | u64`.
  ;; @context(302 f207 f210)
  ;; @context(302 f207 f209)
  (func $conv_u32_to_64 (param $x i32) (result i64)
    (i64.extend_i32_u $x))

  ;; 64-bit identity (`i64 -> i64`, `u64 -> u64`).
  ;; @context(302 f210 f210)
  ;; @context(302 f209 f209)
  (func $conv_id_64 (param $x i64) (result i64)
    (get $x))

  ;; Checked 64-bit narrowing to u8 (`i64 -> u8`, `u64 -> u8`): traps outside 0..=255.
  ;; @context(302 f210 f203)
  ;; @context(302 f209 f203)
  (func $conv_64_to_u8 (param $x i64) (result i32)
    (when (i64.gt_u $x 255) (unreachable))
    (i32.wrap_i64 $x))

  ;; Checked signed narrowing `i64 -> i32`.
  ;; @context(302 f210 f208)
  (func $conv_i64_to_i32 (param $x i64) (result i32)
    (local $res i32)
    (set $res (i32.wrap_i64 $x))
    (when (i64.ne (i64.extend_i32_s $res) $x) (unreachable))
    (get $res))

  ;; Checked unsigned narrowing `u64 -> i32`: traps above i32::MAX.
  ;; @context(302 f209 f208)
  (func $conv_u64_to_i32 (param $x i64) (result i32)
    (when (i64.gt_u $x 0x7FFFFFFF) (unreachable))
    (i32.wrap_i64 $x))

  ;; Checked 64-bit narrowing to u32 (`i64 -> u32`, `u64 -> u32`): traps outside 0..=u32::MAX.
  ;; @context(302 f210 f207)
  ;; @context(302 f209 f207)
  (func $conv_64_to_u32 (param $x i64) (result i32)
    (when (i64.gt_u $x 0xFFFFFFFF) (unreachable))
    (i32.wrap_i64 $x))

  ;; Checked 64-bit sign-change (`i64 -> u64`, `u64 -> i64`): traps when sign bit is set.
  ;; @context(302 f210 f209)
  ;; @context(302 f209 f210)
  (func $conv_64_nonneg (param $x i64) (result i64)
    (when (i64.lt_s $x 0) (unreachable))
    (get $x))
)
