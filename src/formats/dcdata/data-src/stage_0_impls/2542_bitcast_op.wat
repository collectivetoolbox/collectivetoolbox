;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2542 ("Bitcast value operation").
;;
;; Lossless equal-width bit-pattern reinterpretation across `{u8, i32, u32, i64, u64}`,
;; indexed via `Dc 303` ("Type transmutation"): `@context(303 <src> <dst>)`.
(module
  ;; 8-bit (`u8 -> u8`)
  ;; @context(303 f203 f203)
  (func $bitcast_8 (param $x i32) (result i32)
    (i32.and $x 255))

  ;; 32-bit (`i32 <-> u32`)
  ;; @context(303 f208 f208)
  ;; @context(303 f208 f207)
  ;; @context(303 f207 f207)
  ;; @context(303 f207 f208)
  (func $bitcast_32 (param $x i32) (result i32)
    (get $x))

  ;; 64-bit (`i64 <-> u64`)
  ;; @context(303 f210 f210)
  ;; @context(303 f210 f209)
  ;; @context(303 f209 f209)
  ;; @context(303 f209 f210)
  (func $bitcast_64 (param $x i64) (result i64)
    (get $x))
)
