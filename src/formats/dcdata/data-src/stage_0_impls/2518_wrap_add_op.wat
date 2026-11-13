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

;; Stage 0 implementation of Dc 2518 ("Wrapping add operation").
;;
;; Modular two's-complement addition wrapping modulo 2^width without trapping.
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $wrap_add_u8 (param $a i32) (param $b i32) (result i32)
    (i32.and (i32.add $a $b) 255))

  ;; i32 (`f208`) and u32 (`f207`)
  ;; @context(f208)
  ;; @context(f207)
  (func $wrap_add_32 (param $a i32) (param $b i32) (result i32)
    (i32.add $a $b))

  ;; i64 (`f210`) and u64 (`f209`)
  ;; @context(f210)
  ;; @context(f209)
  (func $wrap_add_64 (param $a i64) (param $b i64) (result i64)
    (i64.add $a $b))
)
