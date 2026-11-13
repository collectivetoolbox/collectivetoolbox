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

;; Stage 0 implementation of Dc 2585 ("StageL buffer freeze").
;;
;; Zero-copy O(1) conversion from mutable `buf/T` (`flags = 0`) to immutable
;; `l/T` (`flags = 1`); traps if `$buf` was already frozen.
(module
  ;; buf/* (`2577 275`)
  ;; @context(2577 275)
  (func $buf_freeze (param $buf i32) (result i32)
    (when (i32.ne (i32.load offset=12 $buf) 0) (unreachable))
    (i32.store offset=12 $buf 1)
    (get $buf))
)
