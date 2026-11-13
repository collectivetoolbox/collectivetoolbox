;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 2532 ("Logical NOT operation").
(module
  ;; b (`f1424`)
  ;; @context(f1424)
  (func $logical_not (param $a i32) (result i32)
    (i32.eqz $a))
)
