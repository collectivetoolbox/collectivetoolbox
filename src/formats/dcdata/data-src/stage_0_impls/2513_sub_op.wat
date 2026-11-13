;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2513 ("Subtract operation").
;;
;; Checked integer subtraction across StageL fixed-width integer types; traps
;; via `unreachable` on unsigned borrow or signed two's-complement overflow.
(module
  ;; u8 (`f203`): canonical values reside in 0..=255.
  ;; @context(f203)
  (func $sub_u8 (param $a i32) (param $b i32) (result i32)
    (set $a (i32.and $a 255))
    (set $b (i32.and $b 255))
    (when (i32.lt_u $a $b) (unreachable))
    (i32.sub $a $b))

  ;; u32 (`f207`): unsigned underflow occurs iff a <u b.
  ;; @context(f207)
  (func $sub_u32 (param $a i32) (param $b i32) (result i32)
    (when (i32.lt_u $a $b) (unreachable))
    (i32.sub $a $b))

  ;; i32 (`f208`): signed overflow occurs iff a and b have opposite signs and result differs from a.
  ;; @context(f208)
  (func $sub_i32 (param $a i32) (param $b i32) (result i32)
    (local $res i32)
    (set $res (i32.sub $a $b))
    (when (i32.lt_s (i32.and (i32.xor $a $b) (i32.xor $res $a)) 0)
      (unreachable))
    (get $res))

  ;; u64 (`f209`): unsigned underflow occurs iff a <u b.
  ;; @context(f209)
  (func $sub_u64 (param $a i64) (param $b i64) (result i64)
    (when (i64.lt_u $a $b) (unreachable))
    (i64.sub $a $b))

  ;; i64 (`f210`): signed overflow occurs iff a and b have opposite signs and result differs from a.
  ;; @context(f210)
  (func $sub_i64 (param $a i64) (param $b i64) (result i64)
    (local $res i64)
    (set $res (i64.sub $a $b))
    (when (i64.lt_s (i64.and (i64.xor $a $b) (i64.xor $res $a)) 0)
      (unreachable))
    (get $res))
)
