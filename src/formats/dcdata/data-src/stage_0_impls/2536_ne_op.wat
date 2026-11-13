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

;; Stage 0 implementation of Dc 2536 ("Not equal comparison").
;;
;; Value inequality across StageL booleans, fixed-width integers, and flat
;; immutable sequences (`l/u8`, `l/b`, `l/i32`, `l/u32`, `l/i64`, `l/u64`).
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $ne_u8 (param $a i32) (param $b i32) (result i32)
    (i32.ne (i32.and $a 255) (i32.and $b 255)))

  ;; b (`f1424`), i32 (`f208`), and u32 (`f207`)
  ;; @context(f1424)
  ;; @context(f208)
  ;; @context(f207)
  (func $ne_32 (param $a i32) (param $b i32) (result i32)
    (i32.ne $a $b))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $ne_64 (param $a i64) (param $b i64) (result i32)
    (i64.ne $a $b))

  ;; l/u8 (`2576 f203`) and l/b (`2576 f1424`): 1-byte element structural inequality.
  ;; @context(2576 f203)
  ;; @context(2576 f1424)
  (func $ne_list_1b (param $a i32) (param $b i32) (result i32)
    (local $len i32)
    (local $pa  i32)
    (local $pb  i32)
    (local $i   i32)
    (when (i32.eq $a $b) (return (i32.const 0)))
    (set $len (i32.load offset=4 $a))
    (when (i32.ne $len (i32.load offset=4 $b)) (return (i32.const 1)))
    (set $pa (i32.load $a))
    (set $pb (i32.load $b))
    (set $i 0)
    (block $done
      (loop $cmp
        (br_if $done (i32.eq $i $len))
        (when
          (i32.ne
            (i32.load8_u (i32.add $pa $i))
            (i32.load8_u (i32.add $pb $i)))
          (return (i32.const 1)))
        (set $i (i32.add $i 1))
        (br $cmp)))
    (i32.const 0))

  ;; l/i32 (`2576 f208`) and l/u32 (`2576 f207`): 4-byte element structural inequality.
  ;; @context(2576 f208)
  ;; @context(2576 f207)
  (func $ne_list_4b (param $a i32) (param $b i32) (result i32)
    (local $bytes i32)
    (local $pa    i32)
    (local $pb    i32)
    (local $off   i32)
    (when (i32.eq $a $b) (return (i32.const 0)))
    (set $bytes (i32.load offset=4 $a))
    (when (i32.ne $bytes (i32.load offset=4 $b)) (return (i32.const 1)))
    (set $bytes (i32.shl $bytes 2))
    (set $pa (i32.load $a))
    (set $pb (i32.load $b))
    (set $off 0)
    (block $done
      (loop $cmp
        (br_if $done (i32.eq $off $bytes))
        (when
          (i32.ne
            (i32.load (i32.add $pa $off))
            (i32.load (i32.add $pb $off)))
          (return (i32.const 1)))
        (set $off (i32.add $off 4))
        (br $cmp)))
    (i32.const 0))

  ;; l/i64 (`2576 f210`) and l/u64 (`2576 f209`): 8-byte element structural inequality.
  ;; @context(2576 f210)
  ;; @context(2576 f209)
  (func $ne_list_8b (param $a i32) (param $b i32) (result i32)
    (local $bytes i32)
    (local $pa    i32)
    (local $pb    i32)
    (local $off   i32)
    (when (i32.eq $a $b) (return (i32.const 0)))
    (set $bytes (i32.load offset=4 $a))
    (when (i32.ne $bytes (i32.load offset=4 $b)) (return (i32.const 1)))
    (set $bytes (i32.shl $bytes 3))
    (set $pa (i32.load $a))
    (set $pb (i32.load $b))
    (set $off 0)
    (block $done
      (loop $cmp
        (br_if $done (i32.eq $off $bytes))
        (when
          (i64.ne
            (i64.load (i32.add $pa $off))
            (i64.load (i32.add $pb $off)))
          (return (i32.const 1)))
        (set $off (i32.add $off 8))
        (br $cmp)))
    (i32.const 0))
)
