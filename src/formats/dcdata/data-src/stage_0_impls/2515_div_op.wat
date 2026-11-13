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

;; Stage 0 implementation of Dc 2515 ("Divide operation").
;;
;; Checked truncated integer division; WebAssembly `div_u` and `div_s` natively
;; trap on division by zero and signed `MIN / -1` overflow.
(module
  ;; u8 (`f203`)
  ;; @context(f203)
  (func $div_u8 (param $a i32) (param $b i32) (result i32)
    (i32.div_u (i32.and $a 255) (i32.and $b 255)))

  ;; u32 (`f207`)
  ;; @context(f207)
  (func $div_u32 (param $a i32) (param $b i32) (result i32)
    (i32.div_u $a $b))

  ;; i32 (`f208`)
  ;; @context(f208)
  (func $div_i32 (param $a i32) (param $b i32) (result i32)
    (i32.div_s $a $b))

  ;; u64 (`f209`)
  ;; @context(f209)
  (func $div_u64 (param $a i64) (param $b i64) (result i64)
    (i64.div_u $a $b))

  ;; i64 (`f210`)
  ;; @context(f210)
  (func $div_i64 (param $a i64) (param $b i64) (result i64)
    (i64.div_s $a $b))
)
