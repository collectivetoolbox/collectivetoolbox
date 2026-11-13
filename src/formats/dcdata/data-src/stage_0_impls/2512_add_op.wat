;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2512 ("Add operation").
;;
;; Checked integer addition across StageL fixed-width integer types; traps via
;; `unreachable` on unsigned carry or signed two's-complement overflow.
(module
  ;; u8 (`f203`): canonical values reside in 0..=255.
  ;; @context(f203)
  (func $add_u8 (param $a i32) (param $b i32) (result i32)
    (local $res i32)
    (set $res (i32.add (i32.and $a 255) (i32.and $b 255)))
    (when (i32.gt_u $res 255) (unreachable))
    (get $res))

  ;; u32 (`f207`): unsigned wrap occurs iff (a + b) <u a.
  ;; @context(f207)
  (func $add_u32 (param $a i32) (param $b i32) (result i32)
    (local $res i32)
    (set $res (i32.add $a $b))
    (when (i32.lt_u $res $a) (unreachable))
    (get $res))

  ;; i32 (`f208`): signed overflow occurs iff operands share a sign and differ from result.
  ;; @context(f208)
  (func $add_i32 (param $a i32) (param $b i32) (result i32)
    (local $res i32)
    (set $res (i32.add $a $b))
    (when (i32.lt_s (i32.and (i32.xor $res $a) (i32.xor $res $b)) 0)
      (unreachable))
    (get $res))

  ;; u64 (`f209`): unsigned wrap occurs iff (a + b) <u a.
  ;; @context(f209)
  (func $add_u64 (param $a i64) (param $b i64) (result i64)
    (local $res i64)
    (set $res (i64.add $a $b))
    (when (i64.lt_u $res $a) (unreachable))
    (get $res))

  ;; i64 (`f210`): signed overflow occurs iff operands share a sign and differ from result.
  ;; @context(f210)
  (func $add_i64 (param $a i64) (param $b i64) (result i64)
    (local $res i64)
    (set $res (i64.add $a $b))
    (when (i64.lt_s (i64.and (i64.xor $res $a) (i64.xor $res $b)) 0)
      (unreachable))
    (get $res))
)
