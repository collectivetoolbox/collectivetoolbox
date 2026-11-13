;; SPDX-License-Identifier: AGPL-3.0-or-later
;;
;; This file is part of Collective Toolbox, a database and document workspace and utilities.
;; Copyright (C) 2026 Collective Toolbox Developers
;; Contact: info@collectivetoolbox.com
;;
;; This program is free software: you can redistribute it and/or modify it under
;; the terms of the GNU Affero General Public License as published by the Free
;; Software Foundation, either version 3 of the License, or (at your option) any
;; later version.
;;
;; This program is distributed in the hope that it will be useful, but WITHOUT ANY
;; WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
;; A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
;;
;; You should have received a copy of the GNU Affero General Public License along
;; with this program.  If not, see <https://www.gnu.org/licenses/>.

;; Stage 0 implementation of Dc 2514 ("Multiply operation").
;;
;; Checked integer multiplication across StageL fixed-width integer types; traps
;; via `unreachable` on unsigned or signed overflow.
(module
  ;; u8 (`f203`): 255 * 255 = 65025 fits in i32 without wrapping.
  ;; @context(f203)
  (func $mul_u8 (param $a i32) (param $b i32) (result i32)
    (local $res i32)
    (set $res (i32.mul (i32.and $a 255) (i32.and $b 255)))
    (when (i32.gt_u $res 255) (unreachable))
    (get $res))

  ;; u32 (`f207`): widen to u64 and check upper 32 bits.
  ;; @context(f207)
  (func $mul_u32 (param $a i32) (param $b i32) (result i32)
    (local $prod i64)
    (set $prod (i64.mul (i64.extend_i32_u $a) (i64.extend_i32_u $b)))
    (when (i64.gt_u $prod 0xFFFFFFFF) (unreachable))
    (i32.wrap_i64 $prod))

  ;; i32 (`f208`): widen to i64 and verify sign-extended wrap-back matches.
  ;; @context(f208)
  (func $mul_i32 (param $a i32) (param $b i32) (result i32)
    (local $prod i64)
    (local $res  i32)
    (set $prod (i64.mul (i64.extend_i32_s $a) (i64.extend_i32_s $b)))
    (set $res (i32.wrap_i64 $prod))
    (when (i64.ne (i64.extend_i32_s $res) $prod) (unreachable))
    (get $res))

  ;; u64 (`f209`): when a != 0, unsigned overflow occurs iff (a * b) /u a != b.
  ;; @context(f209)
  (func $mul_u64 (param $a i64) (param $b i64) (result i64)
    (local $res i64)
    (set $res (i64.mul $a $b))
    (when (i64.ne $a 0)
      (when (i64.ne (i64.div_u $res $a) $b) (unreachable)))
    (get $res))

  ;; i64 (`f210`): when a != 0, `i64.div_s` traps on (MIN, -1) and differs from b on all other overflows.
  ;; @context(f210)
  (func $mul_i64 (param $a i64) (param $b i64) (result i64)
    (local $res i64)
    (set $res (i64.mul $a $b))
    (when (i64.ne $a 0)
      (when (i64.ne (i64.div_s $res $a) $b) (unreachable)))
    (get $res))
)
