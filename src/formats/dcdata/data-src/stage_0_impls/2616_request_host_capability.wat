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

;; Stage 0 implementation of Dc 2616 ("Request host capability").
;;
;; Writes the `l/u8` request payload slice `(ptr, len)` to the output mailbox
;; `mem[8..16]`, yields via the reserved `__ctb_capability_yield` suspend stub,
;; reads the host-allocated response slice `(resp_ptr, resp_len)` from
;; `mem[0..8]`, and returns a freshly allocated immutable `l/u8` slice header.
(module
  (memory (export "memory") 1)

  (const $MB_IN_PTR 0)
  (const $MB_IN_LEN 4)
  (const $HEAP_MIN  256)
  (const $MAX_ADDR  0x7FFF0000)

  (global $__heap_ptr (mut i32) (i32.const 256))

  (func $_alloc (param $size i32) (result i32)
    (local $floor i32)
    (local $ptr   i32)
    (local $next  i32)
    (local $cap   i32)
    (when (i32.lt_s $size 0) (unreachable))
    (set $floor (i32.add (i32.load $MB_IN_PTR) (i32.load $MB_IN_LEN)))
    (when (i32.lt_u $floor $HEAP_MIN)
      (set $floor $HEAP_MIN))
    (set $ptr $__heap_ptr)
    (when (i32.lt_u $ptr $floor)
      (set $ptr $floor))
    (set $ptr (i32.and (i32.add $ptr 7) -8))
    (when (i32.gt_u $size (i32.sub $MAX_ADDR $ptr)) (unreachable))
    (set $next (i32.and (i32.add (i32.add $ptr $size) 7) -8))
    (set $cap (i32.shl (memory.size) 16))
    (when (i32.gt_u $next $cap)
      (when
        (i32.eq
          (memory.grow (i32.shr_u (i32.add (i32.sub $next $cap) 65535) 16))
          -1)
        (unreachable)))
    (global.set $__heap_ptr $next)
    (get $ptr))

  (func $__ctb_capability_yield (export "__ctb_capability_yield")
    (nop))

  ;; l/u8 (`2576 f203`)
  ;; @context(2576 f203)
  (func $request_capability (param $req i32) (result i32)
    (local $hdr      i32)
    (local $resp_ptr i32)
    (local $resp_len i32)
    (i32.store (i32.const 8) (i32.load $req))
    (i32.store (i32.const 12) (i32.load offset=4 $req))
    (call $__ctb_capability_yield)
    (set $resp_ptr (i32.load (i32.const 0)))
    (set $resp_len (i32.load (i32.const 4)))
    (set $hdr (call $_alloc (i32.const 16)))
    (i32.store $hdr $resp_ptr)
    (i32.store offset=4 $hdr $resp_len)
    (i32.store offset=8 $hdr $resp_len)
    (i32.store offset=12 $hdr (i32.const 1))
    (get $hdr))
)
