;; SPDX-License-Identifier: AGPL-3.0-or-later
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
