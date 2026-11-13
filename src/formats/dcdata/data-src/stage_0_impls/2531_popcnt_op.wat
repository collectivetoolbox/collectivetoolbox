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

;; Stage 0 implementation of Dc 2531 ("Population count operation").
;;
;; Counts set one-bits in a fixed-width integer and returns a u32 (`i32`).
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $popcnt_u8 (param $a i32) (result i32)
    (i32.popcnt (i32.and $a 255)))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $popcnt_32 (param $a i32) (result i32)
    (i32.popcnt $a))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $popcnt_64 (param $a i64) (result i32)
    (i32.wrap_i64 (i64.popcnt $a)))
)
