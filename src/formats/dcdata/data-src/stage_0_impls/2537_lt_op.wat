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

;; Stage 0 implementation of Dc 2537 ("Less than comparison").
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $lt_u8 (param $a i32) (param $b i32) (result i32)
    (i32.lt_u (i32.and $a 255) (i32.and $b 255)))

  ;; u32 (`f207`)
  ;; @context(f207)
  (func $lt_u32 (param $a i32) (param $b i32) (result i32)
    (i32.lt_u $a $b))

  ;; i32 (`f208`)
  ;; @context(f208)
  (func $lt_i32 (param $a i32) (param $b i32) (result i32)
    (i32.lt_s $a $b))

  ;; u64 (`f209`)
  ;; @context(f209)
  (func $lt_u64 (param $a i64) (param $b i64) (result i32)
    (i64.lt_u $a $b))

  ;; i64 (`f210`)
  ;; @context(f210)
  (func $lt_i64 (param $a i64) (param $b i64) (result i32)
    (i64.lt_s $a $b))
)
