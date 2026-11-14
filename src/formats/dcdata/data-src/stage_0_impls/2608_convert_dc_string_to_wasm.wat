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

;; Stage 0 implementation of Dc 2608 ("Convert Dc string to WASM").
;;
;; Validates and unescapes a `[string]` (`Dc 260` .. `Dc 261`) invocation frame
;; `[Dc 2608 <string>]` (with optional `Dc 258`..`Dc 259` parameter framing) from
;; the linear-memory input mailbox under active task resource quotas, writing the
;; unescaped target Document Character stream in canonical DcTF to the output mailbox.
(module
  (memory (export "memory") 1)

  ;; Linear-memory mailbox header offsets (bytes 0..16)
  (const $MB_IN_PTR  0)
  (const $MB_IN_LEN  4)
  (const $MB_OUT_PTR 8)
  (const $MB_OUT_LEN 12)

  ;; Global DcChar codepoints (`SHORT_DC_REGION_START = 1_114_112` + short ID)
  (const $DC_ESCAPE                   1114367) ;; Dc 255 ("Ignore following Dc")
  (const $DC_BEGIN_PARAMETER          1114370) ;; Dc 258 ("Begin parameter")
  (const $DC_END_PARAMETER            1114371) ;; Dc 259 ("End parameter")
  (const $DC_BEGIN_LITERAL            1114372) ;; Dc 260 ("Begin literal value")
  (const $DC_END_LITERAL              1114373) ;; Dc 261 ("End literal value")
  (const $DC_BEGIN_TYPE_DEF           1114374) ;; Dc 262 ("Begin type definition")
  (const $DC_END_TYPE_DEF             1114375) ;; Dc 263 ("End type definition")
  (const $DC_CONVERT_STRING_TO_WASM   1116720) ;; Dc 2608 ("Convert Dc string to WASM")

  ;; Decoded codepoint for 1..=4-byte DcTF tokens, or -1 for >=5-byte tokens.
  (global $tok_cp (mut i32) (i32.const 0))

  ;; Validates and decodes one canonical DcTF token starting at `$ptr` within
  ;; `[$ptr .. $end)`, storing its 21-bit codepoint (or -1 if >= 5 bytes) in
  ;; `$tok_cp` and returning its byte length (`1..=23`). Traps on malformed,
  ;; truncated, or non-canonical encodings.
  (func $next_dctf (param $ptr i32) (param $end i32) (result i32)
    (local $b0      i32)
    (local $b1      i32)
    (local $b2      i32)
    (local $k       i32)
    (local $len     i32)
    (local $idx     i32)
    (local $cont    i32)
    (local $val     i64)
    (local $min_val i64)

    (when (i32.ge_u $ptr $end) (unreachable))
    (set $b0 (i32.load8_u $ptr))
    (when (i32.lt_u $b0 0x80)
      (global.set $tok_cp $b0)
      (return 1))
    (when (i32.lt_u $b0 0xC2) (unreachable))

    (when (i32.eq $b0 0xFF)
      (when (i32.gt_u (i32.add $ptr 2) $end) (unreachable))
      (set $b1 (i32.load8_u offset=1 $ptr))
      (when (i32.ne (i32.and $b1 0xC0) 0x80) (unreachable))
      (set $k (i32.and (i32.shr_u $b1 2) 0x0F))
      (set $len (i32.add 8 $k))
      (when (i32.gt_u (i32.add $ptr $len) $end) (unreachable))
      (set $idx 2)
      (block $ext_done
        (loop $ext_loop
          (br_if $ext_done (i32.ge_u $idx $len))
          (set $cont (i32.load8_u (i32.add $ptr $idx)))
          (when (i32.ne (i32.and $cont 0xC0) 0x80) (unreachable))
          (set $idx (i32.add $idx 1))
          (br $ext_loop)))
      (set $b2 (i32.load8_u offset=2 $ptr))
      (if (i32.eqz $k)
        (then
          (when (i32.eqz (i32.and $b1 0x03)) (unreachable)))
        (else
          (when
            (i32.and
              (i32.eqz (i32.and $b1 0x03))
              (i32.eqz (i32.and $b2 0x3C)))
            (unreachable))))
      (global.set $tok_cp -1)
      (return $len))

    (if (i32.eq (i32.and $b0 0xE0) 0xC0)
      (then
        (set $len 2)
        (set $val (i64.extend_i32_u (i32.and $b0 0x1F)))
        (set $min_val 0x80))
      (else
        (if (i32.eq (i32.and $b0 0xF0) 0xE0)
          (then
            (set $len 3)
            (set $val (i64.extend_i32_u (i32.and $b0 0x0F)))
            (set $min_val 0x800))
          (else
            (if (i32.eq (i32.and $b0 0xF8) 0xF0)
              (then
                (set $len 4)
                (set $val (i64.extend_i32_u (i32.and $b0 0x07)))
                (set $min_val 0x10000))
              (else
                (if (i32.eq (i32.and $b0 0xFC) 0xF8)
                  (then
                    (set $len 5)
                    (set $val (i64.extend_i32_u (i32.and $b0 0x03)))
                    (set $min_val 0x200000))
                  (else
                    (if (i32.eq (i32.and $b0 0xFE) 0xFC)
                      (then
                        (set $len 6)
                        (set $val (i64.extend_i32_u (i32.and $b0 0x01)))
                        (set $min_val 0x4000000))
                      (else
                        (set $len 7)
                        (set $val 0)
                        (set $min_val 0x80000000)))))))))))

    (when (i32.gt_u (i32.add $ptr $len) $end) (unreachable))
    (set $idx 1)
    (block $cont_done
      (loop $cont_loop
        (br_if $cont_done (i32.ge_u $idx $len))
        (set $cont (i32.load8_u (i32.add $ptr $idx)))
        (when (i32.ne (i32.and $cont 0xC0) 0x80) (unreachable))
        (set $val
          (i64.or
            (i64.shl $val 6)
            (i64.extend_i32_u (i32.and $cont 0x3F))))
        (set $idx (i32.add $idx 1))
        (br $cont_loop)))

    (when (i64.lt_u $val $min_val) (unreachable))
    (if (i32.le_u $len 4)
      (then (global.set $tok_cp (i32.wrap_i64 $val)))
      (else (global.set $tok_cp -1)))
    (get $len))

  (func (export "_start") (result i32)
    (local $in_ptr     i32)
    (local $in_len     i32)
    (local $in_end     i32)
    (local $out_start  i32)
    (local $out_ptr    i32)
    (local $cap        i32)
    (local $need_end   i32)
    (local $tok_len    i32)
    (local $has_param  i32)
    (local $type_depth i32)
    (local $type_items i32)

    (set $in_ptr (i32.load $MB_IN_PTR))
    (set $in_len (i32.load $MB_IN_LEN))
    (set $in_end (i32.add $in_ptr $in_len))
    (set $out_start (i32.and (i32.add $in_end 7) -8))
    (set $out_ptr $out_start)

    ;; Unescaped output never exceeds $in_len bytes; grow linear memory if needed.
    (set $need_end (i32.add $out_start $in_len))
    (set $cap (i32.shl (memory.size) 16))
    (when (i32.gt_u $need_end $cap)
      (when
        (i32.eq
          (memory.grow
            (i32.shr_u (i32.add (i32.sub $need_end $cap) 65535) 16))
          -1)
        (unreachable)))

    ;; 1. Leading Dc 2608 ("Convert Dc string to WASM")
    (set $tok_len (call $next_dctf $in_ptr $in_end))
    (when (i32.ne $tok_cp $DC_CONVERT_STRING_TO_WASM) (unreachable))
    (set $in_ptr (i32.add $in_ptr $tok_len))

    ;; 2. Optional Dc 258 ("Begin parameter") wrapper, followed by Dc 260 ("Begin literal value")
    (set $tok_len (call $next_dctf $in_ptr $in_end))
    (if (i32.eq $tok_cp $DC_BEGIN_PARAMETER)
      (then
        (set $has_param 1)
        (set $in_ptr (i32.add $in_ptr $tok_len))
        (set $tok_len (call $next_dctf $in_ptr $in_end)))
      (else
        (set $has_param 0)))
    (when (i32.ne $tok_cp $DC_BEGIN_LITERAL) (unreachable))
    (set $in_ptr (i32.add $in_ptr $tok_len))

    ;; 3. Required `[262:]` type definition header on the literal
    (set $tok_len (call $next_dctf $in_ptr $in_end))
    (when (i32.ne $tok_cp $DC_BEGIN_TYPE_DEF) (unreachable))
    (set $in_ptr (i32.add $in_ptr $tok_len))
    (set $type_depth 1)
    (set $type_items 0)
    (block $type_done
      (loop $type_loop
        (set $tok_len (call $next_dctf $in_ptr $in_end))
        (set $in_ptr (i32.add $in_ptr $tok_len))
        (if (i32.eq $tok_cp $DC_ESCAPE)
          (then
            (set $tok_len (call $next_dctf $in_ptr $in_end))
            (set $in_ptr (i32.add $in_ptr $tok_len))
            (set $type_items (i32.add $type_items 1)))
          (else
            (if (i32.eq $tok_cp $DC_BEGIN_TYPE_DEF)
              (then
                (set $type_depth (i32.add $type_depth 1))
                (set $type_items (i32.add $type_items 1)))
              (else
                (if (i32.eq $tok_cp $DC_END_TYPE_DEF)
                  (then
                    (set $type_depth (i32.sub $type_depth 1))
                    (br_if $type_done (i32.eqz $type_depth)))
                  (else
                    (set $type_items (i32.add $type_items 1))))))))
        (br $type_loop)))
    (when (i32.eqz $type_items) (unreachable))

    ;; 4. Unescape literal payload until unescaped Dc 261 ("End literal value")
    (block $payload_done
      (loop $payload_loop
        (set $tok_len (call $next_dctf $in_ptr $in_end))
        (br_if $payload_done (i32.eq $tok_cp $DC_END_LITERAL))
        (if (i32.eq $tok_cp $DC_ESCAPE)
          (then
            (set $in_ptr (i32.add $in_ptr $tok_len))
            (set $tok_len (call $next_dctf $in_ptr $in_end))
            (memory.copy $out_ptr $in_ptr $tok_len)
            (set $in_ptr (i32.add $in_ptr $tok_len))
            (set $out_ptr (i32.add $out_ptr $tok_len)))
          (else
            (memory.copy $out_ptr $in_ptr $tok_len)
            (set $in_ptr (i32.add $in_ptr $tok_len))
            (set $out_ptr (i32.add $out_ptr $tok_len))))
        (br $payload_loop)))
    ;; Consume closing Dc 261 ("End literal value")
    (set $in_ptr (i32.add $in_ptr $tok_len))

    ;; 5. Optional closing Dc 259 ("End parameter") and end-of-stream check
    (when (i32.ne $has_param 0)
      (set $tok_len (call $next_dctf $in_ptr $in_end))
      (when (i32.ne $tok_cp $DC_END_PARAMETER) (unreachable))
      (set $in_ptr (i32.add $in_ptr $tok_len)))
    (when (i32.ne $in_ptr $in_end) (unreachable))
    (when (i32.eq $out_ptr $out_start) (unreachable))

    (i32.store $MB_OUT_PTR $out_start)
    (i32.store $MB_OUT_LEN (i32.sub $out_ptr $out_start))
    (i32.const 0)
  )
)
