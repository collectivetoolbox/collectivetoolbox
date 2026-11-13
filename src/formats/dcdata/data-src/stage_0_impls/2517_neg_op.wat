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

;; Stage 0 implementation of Dc 2517 ("Negate operation").
;;
;; Checked signed arithmetic negation (`i32`, `i64`); traps on two's-complement MIN.
(module
  ;; i32 (`f208`)
  ;; @context(f208)
  (func $neg_i32 (param $a i32) (result i32)
    (when (i32.eq $a -2147483648) (unreachable))
    (i32.sub 0 $a))

  ;; i64 (`f210`)
  ;; @context(f210)
  (func $neg_i64 (param $a i64) (result i64)
    (when (i64.eq $a -9223372036854775808) (unreachable))
    (i64.sub 0 $a))
)
