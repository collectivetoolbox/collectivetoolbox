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

;; Stage 0 implementation of Dc 2529 ("Count leading zeros operation").
;;
;; Counts leading zero bits in a fixed-width integer and returns a u32 (`i32`).
(module
  ;; u8 (`f203`): subtract upper 24 zero bits of the 32-bit lane.
  ;; @context(f203)
  (func $clz_u8 (param $a i32) (result i32)
    (i32.sub (i32.clz (i32.and $a 255)) 24))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $clz_32 (param $a i32) (result i32)
    (i32.clz $a))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $clz_64 (param $a i64) (result i32)
    (i32.wrap_i64 (i64.clz $a)))
)
