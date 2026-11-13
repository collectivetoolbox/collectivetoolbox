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

;; Stage 0 implementation of Dc 2583 ("StageL buffer extend").
;;
;; Bulk-appends all elements of immutable `$list` onto mutable `$buf`; traps if
;; `$buf` has been frozen.
(module
  (memory (export "memory") 1)

  (const $MB_IN_PTR 0)
  (const $MB_IN_LEN 4)
  (const $HEAP_MIN  256)
  (const $MAX_ADDR  0x7FFF0000)

  (global $__heap_ptr (mut i32) (i32.const 256))

  (func $_alloc (param $size i32) (result i32)
    (local $floor i32)
    (local $ptr   i32)
    (local $next  i32)
    (local $cap   i32)
    (when (i32.lt_s $size 0) (unreachable))
    (set $floor (i32.add (i32.load $MB_IN_PTR) (i32.load $MB_IN_LEN)))
    (when (i32.lt_u $floor $HEAP_MIN)
      (set $floor $HEAP_MIN))
    (set $ptr $__heap_ptr)
    (when (i32.lt_u $ptr $floor)
      (set $ptr $floor))
    (set $ptr (i32.and (i32.add $ptr 7) -8))
    (when (i32.gt_u $size (i32.sub $MAX_ADDR $ptr)) (unreachable))
    (set $next (i32.and (i32.add (i32.add $ptr $size) 7) -8))
    (set $cap (i32.shl (memory.size) 16))
    (when (i32.gt_u $next $cap)
      (when
        (i32.eq
          (memory.grow (i32.shr_u (i32.add (i32.sub $next $cap) 65535) 16))
          -1)
        (unreachable)))
    (global.set $__heap_ptr $next)
    (get $ptr))

  ;; 1-byte elements: `u8` (`f203`) and `b` (`f1424`).
  ;; @context(2577 f203)
  ;; @context(2577 f1424)
  (func $buf_extend_1b (param $buf i32) (param $list i32)
    (local $add_len  i32)
    (local $len      i32)
    (local $cap      i32)
    (local $data     i32)
    (local $need64   i64)
    (local $need     i32)
    (local $new_cap  i32)
    (local $new_data i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (set $add_len (i32.load offset=4 $list))
    (when (i32.gt_u $add_len 0)
      (set $len (i32.load offset=4 $buf))
      (set $cap (i32.load offset=8 $buf))
      (set $data (i32.load $buf))
      (set $need64 (i64.add (i64.extend_i32_u $len) (i64.extend_i32_u $add_len)))
      (when (i64.gt_u $need64 0x7FFEFF00) (unreachable))
      (set $need (i32.wrap_i64 $need64))
      (when (i32.gt_u $need $cap)
        (if (i32.gt_u $cap 0x3FFF7F80)
          (then (set $new_cap 0x7FFEFF00))
          (else (set $new_cap (i32.shl $cap 1))))
        (when (i32.lt_u $new_cap $need)
          (set $new_cap $need))
        (set $new_data (call $_alloc $new_cap))
        (when (i32.gt_u $len 0)
          (memory.copy $new_data $data $len))
        (i32.store $buf $new_data)
        (i32.store offset=8 $buf $new_cap)
        (set $data $new_data))
      (memory.copy (i32.add $data $len) (i32.load $list) $add_len)
      (i32.store offset=4 $buf $need)))

  ;; 4-byte elements: `i32` (`f208`), `u32` (`f207`), and nested lists `l/*` (`2576 275`).
  ;; @context(2577 f208)
  ;; @context(2577 f207)
  ;; @context(2577 2576 275)
  (func $buf_extend_4b (param $buf i32) (param $list i32)
    (local $add_len  i32)
    (local $len      i32)
    (local $cap      i32)
    (local $data     i32)
    (local $need64   i64)
    (local $need     i32)
    (local $new_cap  i32)
    (local $new_data i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (set $add_len (i32.load offset=4 $list))
    (when (i32.gt_u $add_len 0)
      (set $len (i32.load offset=4 $buf))
      (set $cap (i32.load offset=8 $buf))
      (set $data (i32.load $buf))
      (set $need64 (i64.add (i64.extend_i32_u $len) (i64.extend_i32_u $add_len)))
      (when (i64.gt_u $need64 0x1FFFBFC0) (unreachable))
      (set $need (i32.wrap_i64 $need64))
      (when (i32.gt_u $need $cap)
        (if (i32.gt_u $cap 0x0FFFDFE0)
          (then (set $new_cap 0x1FFFBFC0))
          (else (set $new_cap (i32.shl $cap 1))))
        (when (i32.lt_u $new_cap $need)
          (set $new_cap $need))
        (set $new_data (call $_alloc (i32.shl $new_cap 2)))
        (when (i32.gt_u $len 0)
          (memory.copy $new_data $data (i32.shl $len 2)))
        (i32.store $buf $new_data)
        (i32.store offset=8 $buf $new_cap)
        (set $data $new_data))
      (memory.copy
        (i32.add $data (i32.shl $len 2))
        (i32.load $list)
        (i32.shl $add_len 2))
      (i32.store offset=4 $buf $need)))

  ;; 8-byte elements: `i64` (`f210`) and `u64` (`f209`).
  ;; @context(2577 f210)
  ;; @context(2577 f209)
  (func $buf_extend_8b (param $buf i32) (param $list i32)
    (local $add_len  i32)
    (local $len      i32)
    (local $cap      i32)
    (local $data     i32)
    (local $need64   i64)
    (local $need     i32)
    (local $new_cap  i32)
    (local $new_data i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (set $add_len (i32.load offset=4 $list))
    (when (i32.gt_u $add_len 0)
      (set $len (i32.load offset=4 $buf))
      (set $cap (i32.load offset=8 $buf))
      (set $data (i32.load $buf))
      (set $need64 (i64.add (i64.extend_i32_u $len) (i64.extend_i32_u $add_len)))
      (when (i64.gt_u $need64 0x0FFFDFE0) (unreachable))
      (set $need (i32.wrap_i64 $need64))
      (when (i32.gt_u $need $cap)
        (if (i32.gt_u $cap 0x07FFEF00)
          (then (set $new_cap 0x0FFFDFE0))
          (else (set $new_cap (i32.shl $cap 1))))
        (when (i32.lt_u $new_cap $need)
          (set $new_cap $need))
        (set $new_data (call $_alloc (i32.shl $new_cap 3)))
        (when (i32.gt_u $len 0)
          (memory.copy $new_data $data (i32.shl $len 3)))
        (i32.store $buf $new_data)
        (i32.store offset=8 $buf $new_cap)
        (set $data $new_data))
      (memory.copy
        (i32.add $data (i32.shl $len 3))
        (i32.load $list)
        (i32.shl $add_len 3))
      (i32.store offset=4 $buf $need)))
)
