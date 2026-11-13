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

;; Stage 0 implementation of Dc 2545 ("Slice sequence operation").
;;
;; Zero-copy O(1) sub-sequence view `[start..end)` over an immutable `l/T`
;; sequence; traps unless `start <= end <= len(seq)`.
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
  ;; @context(2576 f203)
  ;; @context(2576 f1424)
  (func $slice_1b (param $seq i32) (param $start i64) (param $end i64) (result i32)
    (local $sub_len i32)
    (local $hdr     i32)
    (when (i64.gt_u $start $end) (unreachable))
    (when (i64.gt_u $end (i64.extend_i32_u (i32.load offset=4 $seq)))
      (unreachable))
    (set $sub_len (i32.wrap_i64 (i64.sub $end $start)))
    (set $hdr (call $_alloc 16))
    (i32.store $hdr (i32.add (i32.load $seq) (i32.wrap_i64 $start)))
    (i32.store offset=4 $hdr $sub_len)
    (i32.store offset=8 $hdr $sub_len)
    (i32.store offset=12 $hdr 1)
    (get $hdr))

  ;; 4-byte elements: `i32` (`f208`), `u32` (`f207`), and nested lists `l/*` (`2576 275`).
  ;; @context(2576 f208)
  ;; @context(2576 f207)
  ;; @context(2576 2576 275)
  (func $slice_4b (param $seq i32) (param $start i64) (param $end i64) (result i32)
    (local $sub_len i32)
    (local $hdr     i32)
    (when (i64.gt_u $start $end) (unreachable))
    (when (i64.gt_u $end (i64.extend_i32_u (i32.load offset=4 $seq)))
      (unreachable))
    (set $sub_len (i32.wrap_i64 (i64.sub $end $start)))
    (set $hdr (call $_alloc 16))
    (i32.store $hdr (i32.add (i32.load $seq) (i32.shl (i32.wrap_i64 $start) 2)))
    (i32.store offset=4 $hdr $sub_len)
    (i32.store offset=8 $hdr $sub_len)
    (i32.store offset=12 $hdr 1)
    (get $hdr))

  ;; 8-byte elements: `i64` (`f210`) and `u64` (`f209`).
  ;; @context(2576 f210)
  ;; @context(2576 f209)
  (func $slice_8b (param $seq i32) (param $start i64) (param $end i64) (result i32)
    (local $sub_len i32)
    (local $hdr     i32)
    (when (i64.gt_u $start $end) (unreachable))
    (when (i64.gt_u $end (i64.extend_i32_u (i32.load offset=4 $seq)))
      (unreachable))
    (set $sub_len (i32.wrap_i64 (i64.sub $end $start)))
    (set $hdr (call $_alloc 16))
    (i32.store $hdr (i32.add (i32.load $seq) (i32.shl (i32.wrap_i64 $start) 3)))
    (i32.store offset=4 $hdr $sub_len)
    (i32.store offset=8 $hdr $sub_len)
    (i32.store offset=12 $hdr 1)
    (get $hdr))
)
