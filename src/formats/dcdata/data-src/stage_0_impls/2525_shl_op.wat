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

;; Stage 0 implementation of Dc 2525 ("Shift left operation").
;;
;; Shifts the first operand left by the u32 bit-count `$cnt`; traps if
;; `$cnt >= bit_width(T)` (never silently masks the shift count).
(module
  ;; u8 (`f203`): width = 8.
  ;; @context(f203)
  (func $shl_u8 (param $a i32) (param $cnt i32) (result i32)
    (when (i32.ge_u $cnt 8) (unreachable))
    (i32.and (i32.shl $a $cnt) 255))

  ;; i32 (`f208`) and u32 (`f207`): width = 32.
  ;; @context(f208)
  ;; @context(f207)
  (func $shl_32 (param $a i32) (param $cnt i32) (result i32)
    (when (i32.ge_u $cnt 32) (unreachable))
    (i32.shl $a $cnt))

  ;; i64 (`f210`) and u64 (`f209`): width = 64 (`$cnt` is u32).
  ;; @context(f210)
  ;; @context(f209)
  (func $shl_64 (param $a i64) (param $cnt i32) (result i64)
    (when (i32.ge_u $cnt 64) (unreachable))
    (i64.shl $a (i64.extend_i32_u $cnt)))
)
