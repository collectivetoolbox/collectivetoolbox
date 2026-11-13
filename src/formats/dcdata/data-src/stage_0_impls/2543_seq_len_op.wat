;; SPDX-License-Identifier: AGPL-3.0-or-later
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
