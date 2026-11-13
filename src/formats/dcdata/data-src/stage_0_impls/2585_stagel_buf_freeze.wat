;; SPDX-License-Identifier: AGPL-3.0-or-later
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
