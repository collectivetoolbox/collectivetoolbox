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
