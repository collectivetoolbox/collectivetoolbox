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

;; Stage 0 implementation of Dc 2584 ("StageL buffer set").
;;
;; Overwrites `$buf[$idx]` with `$val` in O(1) time; traps if `$buf` is frozen
;; or `$idx >= len($buf)`.
(module
  ;; 1-byte elements: `u8` (`f203`) and `b` (`f1424`).
  ;; @context(2577 f203)
  ;; @context(2577 f1424)
  (func $buf_set_1b (param $buf i32) (param $idx i64) (param $val i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $buf)))
      (unreachable))
    (i32.store8 (i32.add (i32.load $buf) (i32.wrap_i64 $idx)) $val))

  ;; 4-byte elements: `i32` (`f208`), `u32` (`f207`), and nested lists `l/*` (`2576 275`).
  ;; @context(2577 f208)
  ;; @context(2577 f207)
  ;; @context(2577 2576 275)
  (func $buf_set_4b (param $buf i32) (param $idx i64) (param $val i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $buf)))
      (unreachable))
    (i32.store (i32.add (i32.load $buf) (i32.shl (i32.wrap_i64 $idx) 2)) $val))

  ;; 8-byte elements: `i64` (`f210`) and `u64` (`f209`).
  ;; @context(2577 f210)
  ;; @context(2577 f209)
  (func $buf_set_8b (param $buf i32) (param $idx i64) (param $val i64)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (when (i64.ge_u $idx (i64.extend_i32_u (i32.load offset=4 $buf)))
      (unreachable))
    (i64.store (i32.add (i32.load $buf) (i32.shl (i32.wrap_i64 $idx) 3)) $val))
)
