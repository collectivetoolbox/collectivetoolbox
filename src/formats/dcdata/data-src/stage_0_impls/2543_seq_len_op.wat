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

;; Stage 0 implementation of Dc 2543 ("Sequence length operation").
;;
;; Reads the 32-bit element count at byte offset +4 of a StageL 16-byte sequence
;; or buffer header (`+0 data_ptr`, `+4 len`, `+8 cap`, `+12 flags`) as `u64`.
(module
  ;; l/* (`2576 275`) and buf/* (`2577 275`)
  ;; @context(2576 275)
  ;; @context(2577 275)
  (func $seq_len (param $seq i32) (result i64)
    (i64.extend_i32_u (i32.load offset=4 $seq)))
)
