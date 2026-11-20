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

;; Self-hosted Stage 0 StageL 3 Token-Dc (`Dc 2192` .. `Dc 2193`) to Dc-WASM
;; (`Dc 1927` .. `Dc 1928`) document transformation (`f1680` `StageL3ToWasm`).
(module
  (memory (export "memory") 1)

  ;; Linear-memory mailbox header offsets (bytes 0..16)
  (const $MB_IN_PTR  0)
  (const $MB_IN_LEN  4)
  (const $MB_OUT_PTR 8)
  (const $MB_OUT_LEN 12)

  ;; Core Document Character codepoints (`1_114_112 + short_id`)
  (const $DC_BEGIN_NUMBER               1114118) ;; Dc 6
  (const $DC_END_NUMBER                 1114119) ;; Dc 7
  (const $DC_NEGATIVE                   1114123) ;; Dc 11
  (const $DC_BASE64_START               1114239) ;; Dc 127
  (const $DC_BASE64_END                 1114302) ;; Dc 190
  (const $DC_BASE64_PADDING             1114307) ;; Dc 195
  (const $DC_START_ENCAP_BINARY         1114315) ;; Dc 203
  (const $DC_END_ENCAP_BINARY           1114316) ;; Dc 204
  (const $DC_BEGIN_SL_COMMENT           1114358) ;; Dc 246
  (const $DC_BEGIN_SL_COMMENT_NL        1114359) ;; Dc 247
  (const $DC_END_SL_COMMENT             1114360) ;; Dc 248
  (const $DC_BEGIN_BLK_COMMENT          1114361) ;; Dc 249
  (const $DC_BEGIN_BLK_COMMENT_NL       1114362) ;; Dc 250
  (const $DC_END_BLK_COMMENT            1114363) ;; Dc 251
  (const $DC_ESCAPE                     1114367) ;; Dc 255
  (const $DC_BEGIN_LITERAL              1114372) ;; Dc 260
  (const $DC_END_LITERAL                1114373) ;; Dc 261
  (const $DC_BEGIN_TYPE_DEF             1114374) ;; Dc 262
  (const $DC_END_TYPE_DEF               1114375) ;; Dc 263
  (const $DC_TYPE_STRING                1114376) ;; Dc 264
  (const $DC_BEGIN_IDENTIFIER           1114382) ;; Dc 270
  (const $DC_END_IDENTIFIER             1114383) ;; Dc 271
  (const $DC_TYPE_ANY                   1114387) ;; Dc 275
  (const $DC_TYPE_ROUTINE               1114392) ;; Dc 280
  (const $DC_TYPE_CONVERSION            1114414) ;; Dc 302
  (const $DC_TYPE_TRANSMUTATION         1114415) ;; Dc 303
  (const $DC_BEGIN_LIST                 1114422) ;; Dc 310
  (const $DC_END_LIST                   1114423) ;; Dc 311
  (const $DC_LIST_SEP                   1114424) ;; Dc 312
  (const $DC_OPTIONAL_ABSENT            1114428) ;; Dc 316
  (const $DC_BEGIN_FORMAT_REPR          1114623) ;; Dc 511
  (const $DC_END_FORMAT_REPR            1114624) ;; Dc 512
  (const $DC_FILE_PATH_DELIM            1114625) ;; Dc 513
  (const $DC_BEGIN_KV_PAIR              1114626) ;; Dc 514
  (const $DC_END_KV_PAIR                1114627) ;; Dc 515
  (const $DC_TYPE_INTERSECTION          1114628) ;; Dc 516
  (const $DC_BOOLEAN_TRUE               1114629) ;; Dc 517
  (const $DC_BOOLEAN_FALSE              1114630) ;; Dc 518
  (const $DC_BEGIN_KV_MAP               1114671) ;; Dc 559
  (const $DC_END_KV_MAP                 1114672) ;; Dc 560
  (const $DC_ENTITY_TYPE                1114677) ;; Dc 565
  (const $DC_EVALUATE_VALUE_AS_ATOM     1114728) ;; Dc 616
  (const $DC_WASM_START                 1114759) ;; Dc 647
  (const $DC_WASM_FB_START              1115015) ;; Dc 903
  (const $DC_WASM_FC_START              1115271) ;; Dc 1159
  (const $DC_WASM_FD_PAGE0_START        1115527) ;; Dc 1415
  (const $DC_BEGIN_WASM_PROGRAM         1116039) ;; Dc 1927
  (const $DC_END_WASM_PROGRAM           1116040) ;; Dc 1928

  ;; StageL 3 structural & expression Document Character codepoints
  (const $DC_BEGIN_MODULE               1116304) ;; Dc 2192
  (const $DC_END_MODULE                 1116305) ;; Dc 2193
  (const $DC_NAMESPACE_DECL             1116306) ;; Dc 2194
  (const $DC_BEGIN_USE_IMPORT           1116307) ;; Dc 2195
  (const $DC_END_USE_IMPORT             1116308) ;; Dc 2196
  (const $DC_USE_IMPORT_ITEM            1116309) ;; Dc 2197
  (const $DC_DECLARE_LET                1116310) ;; Dc 2198
  (const $DC_DECLARE_CONST              1116311) ;; Dc 2199
  (const $DC_DECLARE_PUB                1116312) ;; Dc 2200
  (const $DC_ASSIGN_SET                 1116313) ;; Dc 2201
  (const $DC_TYPED_VALUE_REF            1116314) ;; Dc 2202
  (const $DC_BEGIN_ROUTINE_DECL         1116315) ;; Dc 2203
  (const $DC_END_ROUTINE_DECL           1116316) ;; Dc 2204
  (const $DC_ROUTINE_PARAM              1116317) ;; Dc 2205
  (const $DC_BEGIN_TEST_BLOCK           1116318) ;; Dc 2206
  (const $DC_END_TEST_BLOCK             1116319) ;; Dc 2207
  (const $DC_BEGIN_ROUTINE_CALL         1116323) ;; Dc 2211
  (const $DC_END_ROUTINE_CALL           1116324) ;; Dc 2212
  (const $DC_BEGIN_CODE_BLOCK           1116325) ;; Dc 2213
  (const $DC_END_CODE_BLOCK             1116326) ;; Dc 2214
  (const $DC_BEGIN_TUPLE                1116327) ;; Dc 2215
  (const $DC_END_TUPLE                  1116328) ;; Dc 2216
  (const $DC_BEGIN_POS_STRUCT_DECL      1116329) ;; Dc 2217
  (const $DC_END_POS_STRUCT_DECL        1116330) ;; Dc 2218
  (const $DC_BEGIN_POS_STRUCT_VAL       1116331) ;; Dc 2219
  (const $DC_END_POS_STRUCT_VAL         1116332) ;; Dc 2220
  (const $DC_IF_CONDITIONAL             1116560) ;; Dc 2448
  (const $DC_WHILE_LOOP                 1116561) ;; Dc 2449
  (const $DC_FOR_EACH                   1116562) ;; Dc 2450
  (const $DC_BREAK_LOOP                 1116563) ;; Dc 2451
  (const $DC_CONTINUE_LOOP              1116564) ;; Dc 2452
  (const $DC_RETURN_ROUTINE             1116565) ;; Dc 2453
  (const $DC_FAULT_TRAP                 1116566) ;; Dc 2454
  (const $DC_NO_OP                      1116567) ;; Dc 2455
  (const $DC_ADD_OP                     1116624) ;; Dc 2512
  (const $DC_SUB_OP                     1116625) ;; Dc 2513
  (const $DC_MUL_OP                     1116626) ;; Dc 2514
  (const $DC_DIV_OP                     1116627) ;; Dc 2515
  (const $DC_REM_OP                     1116628) ;; Dc 2516
  (const $DC_NEG_OP                     1116629) ;; Dc 2517
  (const $DC_WRAP_ADD_OP                1116630) ;; Dc 2518
  (const $DC_WRAP_SUB_OP                1116631) ;; Dc 2519
  (const $DC_WRAP_MUL_OP                1116632) ;; Dc 2520
  (const $DC_BIT_AND_OP                 1116633) ;; Dc 2521
  (const $DC_BIT_OR_OP                  1116634) ;; Dc 2522
  (const $DC_BIT_XOR_OP                 1116635) ;; Dc 2523
  (const $DC_BIT_NOT_OP                 1116636) ;; Dc 2524
  (const $DC_SHL_OP                     1116637) ;; Dc 2525
  (const $DC_SHR_OP                     1116638) ;; Dc 2526
  (const $DC_ROTL_OP                    1116639) ;; Dc 2527
  (const $DC_ROTR_OP                    1116640) ;; Dc 2528
  (const $DC_CLZ_OP                     1116641) ;; Dc 2529
  (const $DC_CTZ_OP                     1116642) ;; Dc 2530
  (const $DC_POPCNT_OP                  1116643) ;; Dc 2531
  (const $DC_LOGICAL_NOT_OP             1116644) ;; Dc 2532
  (const $DC_LOGICAL_AND_OP             1116645) ;; Dc 2533
  (const $DC_LOGICAL_OR_OP              1116646) ;; Dc 2534
  (const $DC_EQ_OP                      1116647) ;; Dc 2535
  (const $DC_NE_OP                      1116648) ;; Dc 2536
  (const $DC_LT_OP                      1116649) ;; Dc 2537
  (const $DC_LE_OP                      1116650) ;; Dc 2538
  (const $DC_GT_OP                      1116651) ;; Dc 2539
  (const $DC_GE_OP                      1116652) ;; Dc 2540
  (const $DC_CONVERT_OP                 1116653) ;; Dc 2541
  (const $DC_BITCAST_OP                 1116654) ;; Dc 2542
  (const $DC_SEQ_LEN_OP                 1116655) ;; Dc 2543
  (const $DC_SEQ_GET_OP                 1116656) ;; Dc 2544
  (const $DC_SEQ_SLICE_OP               1116657) ;; Dc 2545
  (const $DC_SEQ_CONCAT_OP              1116658) ;; Dc 2546
  (const $DC_GET_POS_FIELD_OP           1116659) ;; Dc 2547
  (const $DC_WITH_POS_FIELD_OP          1116660) ;; Dc 2548
  (const $DC_STAGEL_LIST_TYPE           1116688) ;; Dc 2576
  (const $DC_STAGEL_BUF_TYPE            1116689) ;; Dc 2577
  (const $DC_BEGIN_STAGEL_LIST          1116690) ;; Dc 2578
  (const $DC_END_STAGEL_LIST            1116691) ;; Dc 2579
  (const $DC_STAGEL_BUF_NEW             1116692) ;; Dc 2580
  (const $DC_STAGEL_BUF_FILL            1116693) ;; Dc 2581
  (const $DC_STAGEL_BUF_PUSH            1116694) ;; Dc 2582
  (const $DC_STAGEL_BUF_EXTEND          1116695) ;; Dc 2583
  (const $DC_STAGEL_BUF_SET             1116696) ;; Dc 2584
  (const $DC_STAGEL_BUF_FREEZE          1116697) ;; Dc 2585
  (const $DC_REQUEST_HOST_CAPABILITY_OP 1116728) ;; Dc 2616
  (const $DC_TYPE_TUPLE                 1116784) ;; Dc 2672
  (const $DC_TYPE_STRUCT                1116785) ;; Dc 2673

  ;; Format Document Character codepoints (`2_228_224 + format_id`)
  (const $FMT_INTEGER                   2228423) ;; f199
  (const $FMT_U8                        2228427) ;; f203
  (const $FMT_U32                       2228431) ;; f207
  (const $FMT_I32                       2228432) ;; f208
  (const $FMT_U64                       2228433) ;; f209
  (const $FMT_I64                       2228434) ;; f210
  (const $FMT_FILENAME                  2228452) ;; f228
  (const $FMT_FILEPATH                  2228453) ;; f229
  (const $FMT_BOOL                      2229648) ;; f1424
  (const $FMT_VOID                      2229649) ;; f1425
  (const $FMT_NEVER                     2229650) ;; f1426

  ;; Canonical interned Type IDs & type kinds
  (const $TY_VOID       0)
  (const $TY_NEVER      1)
  (const $TY_B          2)
  (const $TY_U8         3)
  (const $TY_I32        4)
  (const $TY_U32        5)
  (const $TY_I64        6)
  (const $TY_U64        7)
  (const $TY_TUP0       8)
  (const $TK_LIST       8)
  (const $TK_BUF        9)
  (const $TK_TUPLE      10)
  (const $TK_STRUCT     11)
  (const $TK_RAW_SIDENT 12)
  (const $TK_RAW_SDC    13)

  ;; Expression AST tags (`32`-byte node)
  (const $EX_BOOL       1)
  (const $EX_INT        2)
  (const $EX_BYTES      3)
  (const $EX_LIST       4)
  (const $EX_TUPLE_NEW  5)
  (const $EX_STRUCT_NEW 6)
  (const $EX_FIELD      7)
  (const $EX_WITH       8)
  (const $EX_BUF_NEW    9)
  (const $EX_VALUE_REF  10)
  (const $EX_LET        11)
  (const $EX_CONST      12)
  (const $EX_SET        13)
  (const $EX_ROUTINE    14)
  (const $EX_CALL       15)
  (const $EX_BLOCK      16)
  (const $EX_IF         17)
  (const $EX_WHILE      18)
  (const $EX_FOREACH    19)
  (const $EX_BREAK      20)
  (const $EX_CONTINUE   21)
  (const $EX_RETURN     22)
  (const $EX_FAULT      23)
  (const $EX_NOP        24)
  (const $EX_UNARY      25)
  (const $EX_BINARY     26)
  (const $EX_TERNARY    27)
  (const $EX_CONV       28)
  (const $EX_BITCAST    29)

  (const $SHADOW_STACK_BYTES 65536)
  (const $SHADOW_CELL_BYTES  16)

  ;; Compiler arena & stream state globals
  (global $arena_ptr      (mut i32) (i32.const 0))
  (global $arena_cap      (mut i32) (i32.const 0))
  (global $cur_ptr        (mut i32) (i32.const 0))
  (global $cur_end        (mut i32) (i32.const 0))
  (global $dec_cp         (mut i32) (i32.const 0))
  (global $dec_raw_ptr    (mut i32) (i32.const 0))
  (global $dec_raw_len    (mut i32) (i32.const 0))

  ;; Scratch return registers for multi-value helpers
  (global $ret_ptr        (mut i32) (i32.const 0))
  (global $ret_len        (mut i32) (i32.const 0))
  (global $ret_aux        (mut i32) (i32.const 0))

  ;; Compiler global tables
  (global $builtin_entries (mut i32) (i32.const 0)) ;; $buf of BuiltinEntry*
  (global $out_globals     (mut i32) (i32.const 0)) ;; $buf of GlobalOut*
  (global $out_routines    (mut i32) (i32.const 0)) ;; $buf of RoutineOut*
  (global $alloc_fn_plus1  (mut i32) (i32.const 0))
  (global $type_table      (mut i32) (i32.const 0)) ;; $buf of TypeEntry*
  (global $all_structs     (mut i32) (i32.const 0)) ;; $buf of StructDef*
  (global $modules_buf     (mut i32) (i32.const 0)) ;; $buf of ModAst*
  (global $uses_shadow     (mut i32) (i32.const 0))
  (global $sh_base_glob    (mut i32) (i32.const 0))
  (global $sh_lim_glob     (mut i32) (i32.const 0))
  (global $sh_sp_glob      (mut i32) (i32.const 0))
  (global $sh_push_fn      (mut i32) (i32.const 0))
  (global $has_top_vals    (mut i32) (i32.const 0))
  (global $init_flag_glob  (mut i32) (i32.const 0))
  (global $init_fn_idx     (mut i32) (i32.const 0))
  (global $next_rout_uid   (mut i32) (i32.const 1))

  ;; ---------------------------------------------------------------------------
  ;; 1. Zero-initialized bump arena & dynamic buffers
  ;; ---------------------------------------------------------------------------

  (func $alloc (param $bytes i32) (result i32)
    (local $aligned i32)
    (local $res     i32)
    (local $need    i32)
    (local $pages   i32)
    (when (i32.le_s $bytes 0) (set $bytes 8))
    (set $aligned (i32.and (i32.add $bytes 7) -8))
    (set $res $arena_ptr)
    (set $need (i32.add $res $aligned))
    (when (i32.lt_u $need $res) (unreachable))
    (when (i32.gt_u $need $arena_cap)
      (set $pages (i32.shr_u (i32.add (i32.sub $need $arena_cap) 65535) 16))
      (when (i32.eq (memory.grow $pages) -1) (unreachable))
      (global.set $arena_cap (i32.shl (memory.size) 16)))
    (global.set $arena_ptr $need)
    (memory.fill $res 0 $aligned)
    (get $res))

  (func $slice_eq (param $p1 i32) (param $l1 i32) (param $p2 i32) (param $l2 i32) (result i32)
    (local $i i32)
    (when (i32.ne $l1 $l2) (return 0))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $l1))
        (when
          (i32.ne
            (i32.load8_u (i32.add $p1 $i))
            (i32.load8_u (i32.add $p2 $i)))
          (return 0))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const 1))

  ;; Dynamic buffer (`+0 ptr, +4 len, +8 cap`)
  (func $buf_new (param $cap i32) (result i32)
    (local $b i32)
    (when (i32.lt_s $cap 16) (set $cap 16))
    (set $b (call $alloc 16))
    (i32.store $b (call $alloc $cap))
    (i32.store offset=4 $b 0)
    (i32.store offset=8 $b $cap)
    (get $b))

  (func $buf_reserve (param $b i32) (param $add i32)
    (local $len     i32)
    (local $cap     i32)
    (local $need    i32)
    (local $new_cap i32)
    (local $new_ptr i32)
    (set $len (i32.load offset=4 $b))
    (set $cap (i32.load offset=8 $b))
    (set $need (i32.add $len $add))
    (when (i32.lt_u $need $len) (unreachable))
    (when (i32.le_u $need $cap) (return))
    (set $new_cap (i32.shl $cap 1))
    (when (i32.lt_u $new_cap $need) (set $new_cap (i32.add $need 32)))
    (set $new_ptr (call $alloc $new_cap))
    (when (i32.gt_u $len 0)
      (memory.copy $new_ptr (i32.load $b) $len))
    (i32.store $b $new_ptr)
    (i32.store offset=8 $b $new_cap))

  (func $buf_push_u8 (param $b i32) (param $v i32)
    (local $len i32)
    (call $buf_reserve $b 1)
    (set $len (i32.load offset=4 $b))
    (i32.store8 (i32.add (i32.load $b) $len) $v)
    (i32.store offset=4 $b (i32.add $len 1)))

  (func $buf_push_bytes (param $b i32) (param $src i32) (param $slen i32)
    (local $len i32)
    (when (i32.le_s $slen 0) (return))
    (call $buf_reserve $b $slen)
    (set $len (i32.load offset=4 $b))
    (memory.copy (i32.add (i32.load $b) $len) $src $slen)
    (i32.store offset=4 $b (i32.add $len $slen)))

  (func $buf_push_i32 (param $b i32) (param $v i32)
    (local $len i32)
    (call $buf_reserve $b 4)
    (set $len (i32.load offset=4 $b))
    (i32.store align=1 (i32.add (i32.load $b) $len) $v)
    (i32.store offset=4 $b (i32.add $len 4)))

  (func $buf_count_i32 (param $b i32) (result i32)
    (i32.shr_u (i32.load offset=4 $b) 2))

  (func $buf_get_i32 (param $b i32) (param $idx i32) (result i32)
    (when (i32.ge_u $idx (i32.shr_u (i32.load offset=4 $b) 2)) (unreachable))
    (i32.load align=1 (i32.add (i32.load $b) (i32.shl $idx 2))))

  (func $buf_set_i32 (param $b i32) (param $idx i32) (param $v i32)
    (when (i32.ge_u $idx (i32.shr_u (i32.load offset=4 $b) 2)) (unreachable))
    (i32.store align=1 (i32.add (i32.load $b) (i32.shl $idx 2)) $v))

  (func $buf_push_u32_dec (param $b i32) (param $v i32)
    (local $tmp i32)
    (local $n   i32)
    (when (i32.eqz $v)
      (call $buf_push_u8 $b 48)
      (return))
    (set $tmp (call $alloc 16))
    (set $n 0)
    (block $done
      (loop $lp
        (br_if $done (i32.eqz $v))
        (i32.store8 (i32.add $tmp $n) (i32.add 48 (i32.rem_u $v 10)))
        (set $v (i32.div_u $v 10))
        (set $n (i32.add $n 1))
        (br $lp)))
    (block $edone
      (loop $elp
        (br_if $edone (i32.eqz $n))
        (set $n (i32.sub $n 1))
        (call $buf_push_u8 $b (i32.load8_u (i32.add $tmp $n)))
        (br $elp))))

  ;; Makes a short ASCII identifier `"_<tag>_<num>"`, returning `(ptr, $ret_len)`.
  (func $make_tag_name (param $tag_ch i32) (param $num i32) (result i32)
    (local $b i32)
    (set $b (call $buf_new 16))
    (call $buf_push_u8 $b 95)
    (call $buf_push_u8 $b 95)
    (call $buf_push_u8 $b $tag_ch)
    (call $buf_push_u8 $b 95)
    (call $buf_push_u32_dec $b $num)
    (global.set $ret_len (i32.load offset=4 $b))
    (i32.load $b))

  ;; Makes a fixed ASCII literal from up to 16 packed bytes (`$w0` low 8, `$w1` high 8).
  (func $make_ascii (param $len i32) (param $w0 i64) (param $w1 i64) (result i32)
    (local $p i32)
    (set $p (call $alloc 16))
    (i64.store $p $w0)
    (i64.store offset=8 $p $w1)
    (global.set $ret_len $len)
    (get $p))

  ;; ---------------------------------------------------------------------------
  ;; 2. DcTF (`f641`) Decoder & Canonical Emitter
  ;; ---------------------------------------------------------------------------

  ;; Decodes one canonical DcTF token at `[$ptr .. $end)`.
  ;; Sets `$dec_cp` (`0..0x3FF_FFFF` for 1..=5-byte tokens, `-1` for >=6-byte `Long` Dcs),
  ;; `$dec_raw_ptr = $ptr`, `$dec_raw_len = len`, and returns `len`.
  (func $raw_decode_at (param $ptr i32) (param $end i32) (result i32)
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
    (global.set $dec_raw_ptr $ptr)
    (set $b0 (i32.load8_u $ptr))
    (when (i32.lt_u $b0 0x80)
      (global.set $dec_cp $b0)
      (global.set $dec_raw_len 1)
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
      (global.set $dec_cp -1)
      (global.set $dec_raw_len $len)
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
    (if (i32.le_u $len 5)
      (then (global.set $dec_cp (i32.wrap_i64 $val)))
      (else (global.set $dec_cp -1)))
    (global.set $dec_raw_len $len)
    (get $len))

  (func $raw_peek (result i32)
    (drop (call $raw_decode_at $cur_ptr $cur_end))
    (global.get $dec_cp))

  (func $raw_next (result i32)
    (local $len i32)
    (set $len (call $raw_decode_at $cur_ptr $cur_end))
    (global.set $cur_ptr (i32.add $cur_ptr $len))
    (global.get $dec_cp))

  (func $skip_comment_until (param $closer_cp i32)
    (local $cp i32)
    (block $done
      (loop $lp
        (set $cp (call $raw_next))
        (if (i32.eq $cp $DC_ESCAPE)
          (then (drop (call $raw_next)))
          (else (br_if $done (i32.eq $cp $closer_cp))))
        (br $lp))))

  (func $skip_comments
    (local $cp  i32)
    (local $len i32)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $cur_ptr $cur_end))
        (set $len (call $raw_decode_at $cur_ptr $cur_end))
        (set $cp $dec_cp)
        (if (i32.or (i32.eq $cp $DC_BEGIN_SL_COMMENT) (i32.eq $cp $DC_BEGIN_SL_COMMENT_NL))
          (then
            (global.set $cur_ptr (i32.add $cur_ptr $len))
            (call $skip_comment_until $DC_END_SL_COMMENT)
            (br $lp))
          (else
            (if (i32.or (i32.eq $cp $DC_BEGIN_BLK_COMMENT) (i32.eq $cp $DC_BEGIN_BLK_COMMENT_NL))
              (then
                (global.set $cur_ptr (i32.add $cur_ptr $len))
                (call $skip_comment_until $DC_END_BLK_COMMENT)
                (br $lp)))))
        (br $done))))

  (func $tok_peek (result i32)
    (call $skip_comments)
    (call $raw_peek))

  (func $tok_next (result i32)
    (call $skip_comments)
    (call $raw_next))

  (func $expect_tok (param $expected i32)
    (when (i32.ne (call $tok_next) $expected) (unreachable)))

  ;; Appends canonical 1..=5-byte DcTF encoding of `$cp` (`0 .. 0x3FF_FFFF`) to `$b`.
  (func $emit_cp (param $b i32) (param $cp i32)
    (when (i32.lt_u $cp 0x80)
      (call $buf_push_u8 $b $cp)
      (return))
    (when (i32.le_u $cp 0x7FF)
      (call $buf_push_u8 $b (i32.or 0xC0 (i32.shr_u $cp 6)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and $cp 0x3F)))
      (return))
    (when (i32.le_u $cp 0xFFFF)
      (call $buf_push_u8 $b (i32.or 0xE0 (i32.shr_u $cp 12)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and (i32.shr_u $cp 6) 0x3F)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and $cp 0x3F)))
      (return))
    (when (i32.le_u $cp 0x1FFFFF)
      (call $buf_push_u8 $b (i32.or 0xF0 (i32.shr_u $cp 18)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and (i32.shr_u $cp 12) 0x3F)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and (i32.shr_u $cp 6) 0x3F)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and $cp 0x3F)))
      (return))
    (when (i32.le_u $cp 0x3FFFFFF)
      (call $buf_push_u8 $b (i32.or 0xF8 (i32.shr_u $cp 24)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and (i32.shr_u $cp 18) 0x3F)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and (i32.shr_u $cp 12) 0x3F)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and (i32.shr_u $cp 6) 0x3F)))
      (call $buf_push_u8 $b (i32.or 0x80 (i32.and $cp 0x3F)))
      (return))
    (unreachable))

  ;; Emits `Dc 6 f199 <base64 digits> Dc 7` for unsigned `u64` `$val`.
  (func $emit_u64_dc_num (param $b i32) (param $val i64)
    (local $digits i32)
    (local $n      i32)
    (call $emit_cp $b $DC_BEGIN_NUMBER)
    (call $emit_cp $b $FMT_INTEGER)
    (if (i64.eqz $val)
      (then
        (call $emit_cp $b $DC_BASE64_START))
      (else
        (set $digits (call $alloc 16))
        (set $n 0)
        (block $done
          (loop $lp
            (br_if $done (i64.eqz $val))
            (i32.store8
              (i32.add $digits $n)
              (i32.wrap_i64 (i64.and $val 63)))
            (set $val (i64.shr_u $val 6))
            (set $n (i32.add $n 1))
            (br $lp)))
        (block $edone
          (loop $elp
            (br_if $edone (i32.eqz $n))
            (set $n (i32.sub $n 1))
            (call $emit_cp
              $b
              (i32.add $DC_BASE64_START (i32.load8_u (i32.add $digits $n))))
            (br $elp)))))
    (call $emit_cp $b $DC_END_NUMBER))

  ;; Emits `Dc 6 f199 (Dc 11)? <base64 digits> Dc 7` for signed `i64` `$val`.
  (func $emit_i64_dc_num (param $b i32) (param $val i64)
    (local $mag    i64)
    (local $digits i32)
    (local $n      i32)
    (when (i64.ge_s $val 0)
      (call $emit_u64_dc_num $b $val)
      (return))
    (set $mag (i64.sub 0 $val))
    (call $emit_cp $b $DC_BEGIN_NUMBER)
    (call $emit_cp $b $FMT_INTEGER)
    (call $emit_cp $b $DC_NEGATIVE)
    (set $digits (call $alloc 16))
    (set $n 0)
    (block $done
      (loop $lp
        (br_if $done (i64.eqz $mag))
        (i32.store8
          (i32.add $digits $n)
          (i32.wrap_i64 (i64.and $mag 63)))
        (set $mag (i64.shr_u $mag 6))
        (set $n (i32.add $n 1))
        (br $lp)))
    (block $edone
      (loop $elp
        (br_if $edone (i32.eqz $n))
        (set $n (i32.sub $n 1))
        (call $emit_cp
          $b
          (i32.add $DC_BASE64_START (i32.load8_u (i32.add $digits $n))))
        (br $elp)))
    (call $emit_cp $b $DC_END_NUMBER))

  ;; ---------------------------------------------------------------------------
  ;; 3. Stage 0 `.WASM` Built-in Map Parser & Relocating Linker
  ;; ---------------------------------------------------------------------------

  ;; Skips a balanced `(opener .. closer)` DcTF slice in `[$ptr .. $end)`, honoring `Dc 255`.
  (func $skip_wasm_balanced (param $ptr i32) (param $end i32) (param $opener i32) (param $closer i32) (result i32)
    (local $len   i32)
    (local $cp    i32)
    (local $depth i32)
    (set $len (call $raw_decode_at $ptr $end))
    (when (i32.ne $dec_cp $opener) (unreachable))
    (set $ptr (i32.add $ptr $len))
    (set $depth 1)
    (block $done
      (loop $lp
        (br_if $done (i32.eqz $depth))
        (set $len (call $raw_decode_at $ptr $end))
        (set $cp $dec_cp)
        (set $ptr (i32.add $ptr $len))
        (if (i32.eq $cp $DC_ESCAPE)
          (then
            (set $len (call $raw_decode_at $ptr $end))
            (set $ptr (i32.add $ptr $len)))
          (else
            (if (i32.eq $cp $opener)
              (then (set $depth (i32.add $depth 1)))
              (else
                (when (i32.eq $cp $closer)
                  (set $depth (i32.sub $depth 1)))))))
        (br $lp)))
    (get $ptr))

  (func $skip_wasm_type_expr (param $ptr i32) (param $end i32) (result i32)
    (local $len i32)
    (set $len (call $raw_decode_at $ptr $end))
    (if (i32.eq $dec_cp $DC_BEGIN_TYPE_DEF)
      (then (return (call $skip_wasm_balanced $ptr $end $DC_BEGIN_TYPE_DEF $DC_END_TYPE_DEF)))
      (else (return (i32.add $ptr $len))))
    (unreachable))

  ;; Parses optional leading `Dc 559 Dc 275 Dc 275 [Dc 514 ... Dc 515]* Dc 560` Stage 0 built-in map.
  (func $parse_builtin_map
    (local $cp      i32)
    (local $op_cp   i32)
    (local $ctx_len i32)
    (local $c0      i32)
    (local $c1      i32)
    (local $c2      i32)
    (local $w_start i32)
    (local $w_end   i32)
    (local $entry   i32)

    (when (i32.ne (call $tok_peek) $DC_BEGIN_KV_MAP) (return))
    (drop (call $raw_next)) ;; Dc 559
    (when (i32.ne (call $raw_next) $DC_TYPE_ANY) (unreachable))
    (when (i32.ne (call $raw_next) $DC_TYPE_ANY) (unreachable))

    (block $map_done
      (loop $map_lp
        (set $cp (call $raw_next))
        (br_if $map_done (i32.eq $cp $DC_END_KV_MAP))
        (when (i32.ne $cp $DC_BEGIN_KV_PAIR) (unreachable))
        (when (i32.ne (call $raw_next) $DC_BEGIN_LIST) (unreachable))
        (when (i32.ne (call $raw_next) $DC_TYPE_ANY) (unreachable))
        (set $op_cp (call $raw_next))
        (set $ctx_len 0)
        (set $c0 0)
        (set $c1 0)
        (set $c2 0)
        (block $key_done
          (loop $key_lp
            (set $cp (call $raw_next))
            (br_if $key_done (i32.eq $cp $DC_END_LIST))
            (when (i32.ne $cp $DC_LIST_SEP) (unreachable))
            (set $cp (call $raw_next))
            (if (i32.eqz $ctx_len)
              (then (set $c0 $cp))
              (else
                (if (i32.eq $ctx_len 1)
                  (then (set $c1 $cp))
                  (else
                    (if (i32.eq $ctx_len 2)
                      (then (set $c2 $cp))
                      (else (unreachable)))))))
            (set $ctx_len (i32.add $ctx_len 1))
            (br $key_lp)))
        (when (i32.ne (call $raw_next) $DC_LIST_SEP) (unreachable))
        (set $w_start $cur_ptr)
        (set $w_end (call $skip_wasm_balanced $w_start $cur_end $DC_BEGIN_WASM_PROGRAM $DC_END_WASM_PROGRAM))
        (global.set $cur_ptr $w_end)
        (when (i32.ne (call $raw_next) $DC_END_KV_PAIR) (unreachable))

        ;; BuiltinEntry (32 bytes): +0 op, +4 clen, +8 c0, +12 c1, +16 c2, +20 w_ptr, +24 w_len, +28 fn_plus1
        (set $entry (call $alloc 32))
        (i32.store $entry $op_cp)
        (i32.store offset=4 $entry $ctx_len)
        (i32.store offset=8 $entry $c0)
        (i32.store offset=12 $entry $c1)
        (i32.store offset=16 $entry $c2)
        (i32.store offset=20 $entry $w_start)
        (i32.store offset=24 $entry (i32.sub $w_end $w_start))
        (call $buf_push_i32 $builtin_entries $entry)
        (br $map_lp))))

  ;; Parses `Dc 270 (<key_type>)? <unicode_chars> Dc 271` at `[$ptr .. $end)`.
  ;; Sets `$ret_ptr = name_ptr`, `$ret_len = name_len`, `$ret_aux = kt_slice_ptr` (`+0 kt_ptr, +4 kt_len`),
  ;; and returns `after_ptr`.
  (func $parse_wasm_ident (param $ptr i32) (param $end i32) (result i32)
    (local $len    i32)
    (local $cp     i32)
    (local $kt_ptr i32)
    (local $kt_end i32)
    (local $kt_box i32)
    (local $nbuf   i32)

    (set $len (call $raw_decode_at $ptr $end))
    (when (i32.ne $dec_cp $DC_BEGIN_IDENTIFIER) (unreachable))
    (set $ptr (i32.add $ptr $len))

    (set $kt_ptr $ptr)
    (set $kt_end $ptr)
    (set $len (call $raw_decode_at $ptr $end))
    (set $cp $dec_cp)
    (when
      (i32.and
        (i32.or (i32.lt_s $cp 0) (i32.ge_u $cp 0x110000))
        (i32.and (i32.ne $cp $DC_ESCAPE) (i32.ne $cp $DC_END_IDENTIFIER)))
      (set $kt_end (call $skip_wasm_type_expr $ptr $end))
      (set $ptr $kt_end))

    (set $nbuf (call $buf_new 16))
    (block $done
      (loop $lp
        (set $len (call $raw_decode_at $ptr $end))
        (set $cp $dec_cp)
        (set $ptr (i32.add $ptr $len))
        (br_if $done (i32.eq $cp $DC_END_IDENTIFIER))
        (when (i32.eq $cp $DC_ESCAPE)
          (set $len (call $raw_decode_at $ptr $end))
          (set $cp $dec_cp)
          (set $ptr (i32.add $ptr $len)))
        (when (i32.or (i32.lt_s $cp 0) (i32.ge_u $cp 0x110000)) (unreachable))
        (call $emit_cp $nbuf $cp)
        (br $lp)))

    (set $kt_box (call $alloc 8))
    (i32.store $kt_box $kt_ptr)
    (i32.store offset=4 $kt_box (i32.sub $kt_end $kt_ptr))
    (global.set $ret_ptr (i32.load $nbuf))
    (global.set $ret_len (i32.load offset=4 $nbuf))
    (global.set $ret_aux $kt_box)
    (get $ptr))

  ;; Reads unsigned `u32` Dc number `Dc 6 f199 <base64>+ Dc 7` at `[$ptr .. $end)`.
  ;; Sets `$ret_aux = after_ptr` and returns `u32` value.
  (func $read_wasm_u32_at (param $ptr i32) (param $end i32) (result i32)
    (local $len   i32)
    (local $cp    i32)
    (local $acc   i64)
    (local $ndig  i32)
    (set $len (call $raw_decode_at $ptr $end))
    (when (i32.ne $dec_cp $DC_BEGIN_NUMBER) (unreachable))
    (set $ptr (i32.add $ptr $len))
    (set $len (call $raw_decode_at $ptr $end))
    (when (i32.ne $dec_cp $FMT_INTEGER) (unreachable))
    (set $ptr (i32.add $ptr $len))
    (set $acc 0)
    (set $ndig 0)
    (block $done
      (loop $lp
        (set $len (call $raw_decode_at $ptr $end))
        (set $cp $dec_cp)
        (set $ptr (i32.add $ptr $len))
        (br_if $done (i32.eq $cp $DC_END_NUMBER))
        (when (i32.or (i32.lt_u $cp $DC_BASE64_START) (i32.gt_u $cp $DC_BASE64_END))
          (unreachable))
        (set $acc
          (i64.or
            (i64.shl $acc 6)
            (i64.extend_i32_u (i32.sub $cp $DC_BASE64_START))))
        (when (i64.gt_u $acc 0xFFFFFFFF) (unreachable))
        (set $ndig (i32.add $ndig 1))
        (br $lp)))
    (when (i32.eqz $ndig) (unreachable))
    (global.set $ret_aux $ptr)
    (i32.wrap_i64 $acc))

  (func $copy_wasm_num_at (param $ptr i32) (param $end i32) (param $out i32) (result i32)
    (local $after i32)
    (set $after (call $skip_wasm_balanced $ptr $end $DC_BEGIN_NUMBER $DC_END_NUMBER))
    (call $buf_push_bytes $out $ptr (i32.sub $after $ptr))
    (get $after))

  ;; Relocates `call` (`0x10`), `return_call` (`0x12`), `ref.func` (`0xD2`),
  ;; `global.get` (`0x23`), and `global.set` (`0x24`) inside a `Dc 260..261` `.WASM` routine.
  ;; Returns `(reloc_ptr)` and sets `$ret_len`.
  (func $relocate_wasm_routine
    (param $r_ptr i32) (param $r_len i32) (param $func_remap i32) (param $glob_remap i32) (result i32)
    (local $end      i32)
    (local $body_end i32)
    (local $ptr      i32)
    (local $after    i32)
    (local $len      i32)
    (local $cp       i32)
    (local $sub      i32)
    (local $old_idx  i32)
    (local $new_idx  i32)
    (local $out      i32)

    (set $end (i32.add $r_ptr $r_len))
    (set $len (call $raw_decode_at $r_ptr $end))
    (when (i32.ne $dec_cp $DC_BEGIN_LITERAL) (unreachable))
    (set $ptr (i32.add $r_ptr $len))
    (set $ptr (call $skip_wasm_type_expr $ptr $end))

    (set $len (call $raw_decode_at $ptr $end))
    (when (i32.eq $dec_cp $DC_BEGIN_LIST)
      (set $ptr (call $skip_wasm_balanced $ptr $end $DC_BEGIN_LIST $DC_END_LIST)))

    (set $out (call $buf_new $r_len))
    (call $buf_push_bytes $out $r_ptr (i32.sub $ptr $r_ptr))

    ;; Last token in `[$r_ptr .. $end)` is `Dc 261` (`4` bytes in DcTF)
    (set $body_end (i32.sub $end 4))
    (set $len (call $raw_decode_at $body_end $end))
    (when (i32.ne $dec_cp $DC_END_LITERAL) (unreachable))

    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $ptr $body_end))
        (set $len (call $raw_decode_at $ptr $body_end))
        (set $cp $dec_cp)
        (call $buf_push_bytes $out $ptr $len)
        (set $ptr (i32.add $ptr $len))

        (if (i32.and (i32.ge_u $cp $DC_WASM_START) (i32.lt_u $cp $DC_WASM_FB_START))
          (then
            (set $sub (i32.sub $cp $DC_WASM_START))
            (if (i32.and (i32.ge_u $sub 0x02) (i32.le_u $sub 0x04))
              (then
                (set $after (call $skip_wasm_type_expr $ptr $body_end))
                (call $buf_push_bytes $out $ptr (i32.sub $after $ptr))
                (set $ptr $after))
              (else
                (if (i32.or (i32.or (i32.eq $sub 0x10) (i32.eq $sub 0x12)) (i32.eq $sub 0xD2))
                  (then
                    (set $old_idx (call $read_wasm_u32_at $ptr $body_end))
                    (set $ptr $ret_aux)
                    (set $new_idx (call $buf_get_i32 $func_remap $old_idx))
                    (call $emit_u64_dc_num $out (i64.extend_i32_u $new_idx)))
                  (else
                    (if (i32.or (i32.eq $sub 0x23) (i32.eq $sub 0x24))
                      (then
                        (set $old_idx (call $read_wasm_u32_at $ptr $body_end))
                        (set $ptr $ret_aux)
                        (set $new_idx (call $buf_get_i32 $glob_remap $old_idx))
                        (call $emit_u64_dc_num $out (i64.extend_i32_u $new_idx)))
                      (else
                        (if
                          (i32.or
                            (i32.or (i32.eq $sub 0x0C) (i32.eq $sub 0x0D))
                            (i32.or
                              (i32.and (i32.ge_u $sub 0x20) (i32.le_u $sub 0x22))
                              (i32.and (i32.ge_u $sub 0x3F) (i32.le_u $sub 0x44))))
                          (then
                            (set $ptr (call $copy_wasm_num_at $ptr $body_end $out)))
                          (else
                            (if (i32.eq $sub 0x0E)
                              (then
                                (set $after (call $skip_wasm_balanced $ptr $body_end $DC_BEGIN_LIST $DC_END_LIST))
                                (call $buf_push_bytes $out $ptr (i32.sub $after $ptr))
                                (set $ptr (call $copy_wasm_num_at $after $body_end $out)))
                              (else
                                (if (i32.or (i32.eq $sub 0x11) (i32.eq $sub 0x13))
                                  (then
                                    (set $ptr (call $copy_wasm_num_at $ptr $body_end $out))
                                    (set $ptr (call $copy_wasm_num_at $ptr $body_end $out)))
                                  (else
                                    (if (i32.eq $sub 0x1C)
                                      (then
                                        (set $after (call $skip_wasm_type_expr $ptr $body_end))
                                        (call $buf_push_bytes $out $ptr (i32.sub $after $ptr))
                                        (set $ptr $after))
                                      (else
                                        (when (i32.and (i32.ge_u $sub 0x28) (i32.le_u $sub 0x3E))
                                          (set $ptr (call $copy_wasm_num_at $ptr $body_end $out))
                                          (set $ptr (call $copy_wasm_num_at $ptr $body_end $out))
                                          (set $ptr (call $copy_wasm_num_at $ptr $body_end $out))))))))))))))))))
          (else
            (when (i32.and (i32.ge_u $cp $DC_WASM_FC_START) (i32.lt_u $cp $DC_WASM_FD_PAGE0_START))
              (set $sub (i32.sub $cp $DC_WASM_FC_START))
              (if
                (i32.or
                  (i32.or (i32.eq $sub 8) (i32.eq $sub 10))
                  (i32.or (i32.eq $sub 12) (i32.eq $sub 14)))
                (then
                  (set $ptr (call $copy_wasm_num_at $ptr $body_end $out))
                  (set $ptr (call $copy_wasm_num_at $ptr $body_end $out)))
                (else
                  (when
                    (i32.or
                      (i32.or (i32.eq $sub 9) (i32.eq $sub 11))
                      (i32.or (i32.eq $sub 13) (i32.and (i32.ge_u $sub 15) (i32.le_u $sub 17))))
                    (set $ptr (call $copy_wasm_num_at $ptr $body_end $out))))))))
        (br $lp)))

    (call $emit_cp $out $DC_END_LITERAL)
    (global.set $ret_len (i32.load offset=4 $out))
    (i32.load $out))

  ;; Output GlobalOut (24 bytes): +0 n_ptr, +4 n_len, +8 kt_ptr, +12 kt_len, +16 init_ptr, +20 init_len
  (func $find_global_by_name (param $n_ptr i32) (param $n_len i32) (result i32)
    (local $cnt i32)
    (local $i   i32)
    (local $g   i32)
    (set $cnt (call $buf_count_i32 $out_globals))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $g (call $buf_get_i32 $out_globals $i))
        (when (call $slice_eq (i32.load $g) (i32.load offset=4 $g) $n_ptr $n_len)
          (return $i))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const -1))

  (func $ensure_global_raw
    (param $n_ptr i32) (param $n_len i32) (param $kt_ptr i32) (param $kt_len i32) (param $init_ptr i32) (param $init_len i32) (result i32)
    (local $idx i32)
    (local $g   i32)
    (set $idx (call $find_global_by_name $n_ptr $n_len))
    (when (i32.ge_s $idx 0) (return $idx))
    (set $idx (call $buf_count_i32 $out_globals))
    (set $g (call $alloc 24))
    (i32.store $g $n_ptr)
    (i32.store offset=4 $g $n_len)
    (i32.store offset=8 $g $kt_ptr)
    (i32.store offset=12 $g $kt_len)
    (i32.store offset=16 $g $init_ptr)
    (i32.store offset=20 $g $init_len)
    (call $buf_push_i32 $out_globals $g)
    (get $idx))

  (func $ensure_global_vt (param $n_ptr i32) (param $n_len i32) (param $vt_cp i32) (result i32)
    (local $kt_b   i32)
    (local $init_b i32)
    (local $idx    i32)
    (set $idx (call $find_global_by_name $n_ptr $n_len))
    (when (i32.ge_s $idx 0) (return $idx))
    (set $kt_b (call $buf_new 8))
    (call $emit_cp $kt_b $vt_cp)
    (set $init_b (call $buf_new 16))
    (call $emit_i64_dc_num $init_b 0)
    (call $ensure_global_raw
      $n_ptr $n_len
      (i32.load $kt_b) (i32.load offset=4 $kt_b)
      (i32.load $init_b) (i32.load offset=4 $init_b)))

  ;; Output RoutineOut (16 bytes): +0 n_ptr, +4 n_len, +8 body_ptr, +12 body_len
  (func $find_routine_by_name (param $n_ptr i32) (param $n_len i32) (result i32)
    (local $cnt i32)
    (local $i   i32)
    (local $r   i32)
    (set $cnt (call $buf_count_i32 $out_routines))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $r (call $buf_get_i32 $out_routines $i))
        (when (call $slice_eq (i32.load $r) (i32.load offset=4 $r) $n_ptr $n_len)
          (return $i))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const -1))

  (func $reserve_routine (param $n_ptr i32) (param $n_len i32) (result i32)
    (local $idx i32)
    (local $r   i32)
    (when (i32.ge_s (call $find_routine_by_name $n_ptr $n_len) 0) (unreachable))
    (set $idx (call $buf_count_i32 $out_routines))
    (set $r (call $alloc 16))
    (i32.store $r $n_ptr)
    (i32.store offset=4 $r $n_len)
    (call $buf_push_i32 $out_routines $r)
    (get $idx))

  (func $define_routine (param $idx i32) (param $body_ptr i32) (param $body_len i32)
    (local $r i32)
    (set $r (call $buf_get_i32 $out_routines $idx))
    (when (i32.ne (i32.load offset=12 $r) 0) (unreachable))
    (i32.store offset=8 $r $body_ptr)
    (i32.store offset=12 $r $body_len))

  ;; Finds a Stage 0 `.WASM` built-in entry for `($op_cp, $ctx_len, $c0, $c1, $c2)`.
  ;; Sets `$ret_aux = match_idx` and returns `$matched`.
  (func $find_builtin_entry
    (param $op_cp i32) (param $ctx_len i32) (param $c0 i32) (param $c1 i32) (param $c2 i32) (result i32)
    (local $cnt       i32)
    (local $i         i32)
    (local $e         i32)
    (local $matched   i32)
    (local $match_idx i32)
    (local $best_plen i32)
    (local $eclen     i32)
    (local $last_tok  i32)
    (local $plen      i32)
    (local $ok        i32)

    (set $cnt (call $buf_count_i32 $builtin_entries))
    (set $matched 0)
    (set $match_idx -1)

    ;; Pass 1: exact context match
    (set $i 0)
    (block $p1_done
      (loop $p1_lp
        (br_if $p1_done (i32.ge_u $i $cnt))
        (set $e (call $buf_get_i32 $builtin_entries $i))
        (when
          (i32.and
            (i32.eq (i32.load $e) $op_cp)
            (i32.eq (i32.load offset=4 $e) $ctx_len))
          (set $ok 1)
          (when (i32.and (i32.ge_u $ctx_len 1) (i32.ne (i32.load offset=8 $e) $c0)) (set $ok 0))
          (when (i32.and (i32.ge_u $ctx_len 2) (i32.ne (i32.load offset=12 $e) $c1)) (set $ok 0))
          (when (i32.and (i32.ge_u $ctx_len 3) (i32.ne (i32.load offset=16 $e) $c2)) (set $ok 0))
          (when $ok
            (set $matched $e)
            (set $match_idx $i)
            (br $p1_done)))
        (set $i (i32.add $i 1))
        (br $p1_lp)))

    ;; Pass 2: longest trailing `Dc 275` (`*`) wildcard prefix match
    (when (i32.eqz $matched)
      (set $best_plen 0)
      (set $i 0)
      (block $p2_done
        (loop $p2_lp
          (br_if $p2_done (i32.ge_u $i $cnt))
          (set $e (call $buf_get_i32 $builtin_entries $i))
          (when (i32.eq (i32.load $e) $op_cp)
            (set $eclen (i32.load offset=4 $e))
            (when (i32.ge_u $eclen 2)
              (set $last_tok
                (if (result i32) (i32.eq $eclen 2)
                  (then (i32.load offset=12 $e))
                  (else (i32.load offset=16 $e))))
              (set $plen (i32.sub $eclen 1))
              (when
                (i32.and
                  (i32.eq $last_tok $DC_TYPE_ANY)
                  (i32.and (i32.gt_u $ctx_len $plen) (i32.gt_u $plen $best_plen)))
                (set $ok 1)
                (when (i32.ne (i32.load offset=8 $e) $c0) (set $ok 0))
                (when (i32.and (i32.ge_u $plen 2) (i32.ne (i32.load offset=12 $e) $c1)) (set $ok 0))
                (when $ok
                  (set $best_plen $plen)
                  (set $matched $e)
                  (set $match_idx $i)))))
          (set $i (i32.add $i 1))
          (br $p2_lp))))

    (when (i32.eqz $matched) (unreachable))
    (global.set $ret_aux $match_idx)
    (get $matched))

  ;; Links `$matched` (from `$find_builtin_entry`) into the output module and returns `func_idx`.
  (func $link_builtin_matched (param $matched i32) (param $match_idx i32) (result i32)
    (local $w_ptr      i32)
    (local $w_end      i32)
    (local $in_ptr     i32)
    (local $in_end     i32)
    (local $len        i32)
    (local $cp         i32)
    (local $glob_remap i32)
    (local $func_remap i32)
    (local $pend_idx   i32)
    (local $pend_ptr   i32)
    (local $pend_len   i32)
    (local $n_ptr      i32)
    (local $n_len      i32)
    (local $kt_box     i32)
    (local $after      i32)
    (local $sym_ptr    i32)
    (local $sym_len    i32)
    (local $main_lit   i32)
    (local $is_main    i32)
    (local $f_idx      i32)
    (local $main_idx   i32)
    (local $rel_ptr    i32)
    (local $cnt        i32)
    (local $i          i32)

    (when (i32.gt_u (i32.load offset=28 $matched) 0)
      (return (i32.sub (i32.load offset=28 $matched) 1)))

    (set $sym_ptr (call $make_tag_name 98 $match_idx)) ;; "__b_<match_idx>"
    (set $sym_len $ret_len)

    (set $w_ptr (i32.load offset=20 $matched))
    (set $w_end (i32.add $w_ptr (i32.load offset=24 $matched)))
    ;; Strip outer `Dc 1927` (`4` bytes) and `Dc 1928` (`4` bytes)
    (set $len (call $raw_decode_at $w_ptr $w_end))
    (when (i32.ne $dec_cp $DC_BEGIN_WASM_PROGRAM) (unreachable))
    (set $in_ptr (i32.add $w_ptr $len))
    (set $in_end (i32.sub $w_end 4))
    (set $len (call $raw_decode_at $in_end $w_end))
    (when (i32.ne $dec_cp $DC_END_WASM_PROGRAM) (unreachable))

    ;; Standalone routine `1927 260 ... 261 1928`
    (set $len (call $raw_decode_at $in_ptr $in_end))
    (when (i32.eq $dec_cp $DC_BEGIN_LITERAL)
      (set $f_idx (call $reserve_routine $sym_ptr $sym_len))
      (call $define_routine $f_idx $in_ptr (i32.sub $in_end $in_ptr))
      (i32.store offset=28 $matched (i32.add $f_idx 1))
      (return $f_idx))

    ;; KV-map `559 ... 560`
    (when (i32.ne $dec_cp $DC_BEGIN_KV_MAP) (unreachable))
    (set $in_ptr (i32.add $in_ptr $len))
    (set $in_end (i32.sub $in_end 4))
    (set $len (call $raw_decode_at $in_end (i32.add $in_end 4)))
    (when (i32.ne $dec_cp $DC_END_KV_MAP) (unreachable))

    ;; Optional map length number + keyType + valType
    (set $len (call $raw_decode_at $in_ptr $in_end))
    (when (i32.eq $dec_cp $DC_BEGIN_NUMBER)
      (set $in_ptr (call $skip_wasm_balanced $in_ptr $in_end $DC_BEGIN_NUMBER $DC_END_NUMBER)))
    (set $in_ptr (call $skip_wasm_type_expr $in_ptr $in_end))
    (set $in_ptr (call $skip_wasm_type_expr $in_ptr $in_end))

    (set $glob_remap (call $buf_new 16))
    (set $func_remap (call $buf_new 16))
    (set $pend_idx (call $buf_new 16))
    (set $pend_ptr (call $buf_new 16))
    (set $pend_len (call $buf_new 16))
    (set $main_lit (call $make_ascii 4 0x6E69616D 0)) ;; "main"
    (set $main_idx -1)

    (block $kv_done
      (loop $kv_lp
        (br_if $kv_done (i32.ge_u $in_ptr $in_end))
        (set $len (call $raw_decode_at $in_ptr $in_end))
        (when (i32.ne $dec_cp $DC_BEGIN_KV_PAIR) (unreachable))
        (set $in_ptr (i32.add $in_ptr $len))

        (set $in_ptr (call $parse_wasm_ident $in_ptr $in_end))
        (set $n_ptr $ret_ptr)
        (set $n_len $ret_len)
        (set $kt_box $ret_aux)

        (set $len (call $raw_decode_at $in_ptr $in_end))
        (when (i32.ne $dec_cp $DC_LIST_SEP) (unreachable))
        (set $in_ptr (i32.add $in_ptr $len))

        (set $len (call $raw_decode_at $in_ptr $in_end))
        (set $cp $dec_cp)
        (if (i32.eq $cp $DC_BEGIN_NUMBER)
          (then
            (set $after (call $skip_wasm_balanced $in_ptr $in_end $DC_BEGIN_NUMBER $DC_END_NUMBER))
            (call $buf_push_i32
              $glob_remap
              (call $ensure_global_raw
                $n_ptr $n_len
                (i32.load $kt_box) (i32.load offset=4 $kt_box)
                $in_ptr (i32.sub $after $in_ptr)))
            (set $in_ptr $after))
          (else
            (when (i32.ne $cp $DC_BEGIN_LITERAL) (unreachable))
            (set $after (call $skip_wasm_balanced $in_ptr $in_end $DC_BEGIN_LITERAL $DC_END_LITERAL))
            (set $is_main (call $slice_eq $n_ptr $n_len $main_lit 4))
            (if $is_main
              (then
                (set $n_ptr $sym_ptr)
                (set $n_len $sym_len))
              (else))
            (set $f_idx (call $find_routine_by_name $n_ptr $n_len))
            (when (i32.lt_s $f_idx 0)
              (set $f_idx (call $reserve_routine $n_ptr $n_len))
              (call $buf_push_i32 $pend_idx $f_idx)
              (call $buf_push_i32 $pend_ptr $in_ptr)
              (call $buf_push_i32 $pend_len (i32.sub $after $in_ptr)))
            (when $is_main (set $main_idx $f_idx))
            (call $buf_push_i32 $func_remap $f_idx)
            (set $in_ptr $after)))

        (set $len (call $raw_decode_at $in_ptr $in_end))
        (when (i32.ne $dec_cp $DC_END_KV_PAIR) (unreachable))
        (set $in_ptr (i32.add $in_ptr $len))
        (br $kv_lp)))

    (set $cnt (call $buf_count_i32 $pend_idx))
    (set $i 0)
    (block $rel_done
      (loop $rel_lp
        (br_if $rel_done (i32.ge_u $i $cnt))
        (set $rel_ptr
          (call $relocate_wasm_routine
            (call $buf_get_i32 $pend_ptr $i)
            (call $buf_get_i32 $pend_len $i)
            $func_remap
            $glob_remap))
        (call $define_routine (call $buf_get_i32 $pend_idx $i) $rel_ptr $ret_len)
        (set $i (i32.add $i 1))
        (br $rel_lp)))

    (when (i32.lt_s $main_idx 0) (unreachable))
    (i32.store offset=28 $matched (i32.add $main_idx 1))
    (get $main_idx))

  ;; Matches & links a Stage 0 `.WASM` built-in for `($op_cp, $ctx_len, $c0, $c1, $c2)`.
  (func $link_builtin
    (param $op_cp i32) (param $ctx_len i32) (param $c0 i32) (param $c1 i32) (param $c2 i32) (result i32)
    (local $matched i32)
    (set $matched (call $find_builtin_entry $op_cp $ctx_len $c0 $c1 $c2))
    (call $link_builtin_matched $matched $ret_aux))

  ;; If `$matched` is a pure, relocation-free standalone `[Routine]` beginning with
  ;; `local.get 0` (and `local.get 1` for `$arity == 2`) with no locals, appends its
  ;; trailing instruction tokens directly into `$fb`'s body buffer and returns `1`.
  (func $try_inline_builtin (param $fb i32) (param $matched i32) (param $arity i32) (result i32)
    (local $w_ptr    i32)
    (local $w_len    i32)
    (local $w_end    i32)
    (local $in_ptr   i32)
    (local $in_end   i32)
    (local $body_end i32)
    (local $ptr      i32)
    (local $len      i32)
    (local $k        i32)
    (local $tail_ptr i32)
    (local $scan     i32)
    (local $cp       i32)
    (local $sub      i32)

    (set $w_ptr (i32.load offset=20 $matched))
    (set $w_len (i32.load offset=24 $matched))
    (when (i32.lt_u $w_len 12) (return 0))
    (set $w_end (i32.add $w_ptr $w_len))
    (set $len (call $raw_decode_at $w_ptr $w_end))
    (when (i32.ne $dec_cp $DC_BEGIN_WASM_PROGRAM) (return 0))
    (set $in_ptr (i32.add $w_ptr $len))
    (set $in_end (i32.sub $w_end 4))
    (when (i32.ge_u $in_ptr $in_end) (return 0))
    (set $len (call $raw_decode_at $in_end $w_end))
    (when (i32.ne $dec_cp $DC_END_WASM_PROGRAM) (return 0))

    ;; Only standalone `[Routine]` (`Dc 260 .. Dc 261`) entries qualify
    (set $len (call $raw_decode_at $in_ptr $in_end))
    (when (i32.ne $dec_cp $DC_BEGIN_LITERAL) (return 0))
    (set $ptr (i32.add $in_ptr $len))
    (set $body_end (i32.sub $in_end 4))
    (when (i32.ge_u $ptr $body_end) (return 0))
    (set $len (call $raw_decode_at $body_end $in_end))
    (when (i32.ne $dec_cp $DC_END_LITERAL) (return 0))

    (set $ptr (call $skip_wasm_type_expr $ptr $body_end))
    (when (i32.ge_u $ptr $body_end) (return 0))

    ;; Optional empty locals list (`Dc 310 Dc 565 Dc 311`) only
    (set $len (call $raw_decode_at $ptr $body_end))
    (when (i32.eq $dec_cp $DC_BEGIN_LIST)
      (set $ptr (i32.add $ptr $len))
      (when (i32.ge_u $ptr $body_end) (return 0))
      (set $len (call $raw_decode_at $ptr $body_end))
      (when (i32.ne $dec_cp $DC_ENTITY_TYPE) (return 0))
      (set $ptr (i32.add $ptr $len))
      (when (i32.ge_u $ptr $body_end) (return 0))
      (set $len (call $raw_decode_at $ptr $body_end))
      (when (i32.ne $dec_cp $DC_END_LIST) (return 0))
      (set $ptr (i32.add $ptr $len)))

    ;; Must begin with exact `local.get 0` (and `local.get 1` if binary)
    (set $k 0)
    (block $pg_done
      (loop $pg_lp
        (br_if $pg_done (i32.ge_u $k $arity))
        (when (i32.ge_u $ptr $body_end) (return 0))
        (set $len (call $raw_decode_at $ptr $body_end))
        (when (i32.ne $dec_cp (i32.add $DC_WASM_START 0x20)) (return 0))
        (set $ptr (i32.add $ptr $len))
        (when (i32.ge_u $ptr $body_end) (return 0))
        (drop (call $raw_decode_at $ptr $body_end))
        (when (i32.ne $dec_cp $DC_BEGIN_NUMBER) (return 0))
        (when (i32.ne (call $read_wasm_u32_at $ptr $body_end) $k) (return 0))
        (set $ptr $ret_aux)
        (set $k (i32.add $k 1))
        (br $pg_lp)))

    ;; Verify remaining tail contains only pure, relocation-free core ops
    (set $tail_ptr $ptr)
    (set $scan $tail_ptr)
    (block $sc_done
      (loop $sc_lp
        (br_if $sc_done (i32.ge_u $scan $body_end))
        (set $len (call $raw_decode_at $scan $body_end))
        (set $cp $dec_cp)
        (set $scan (i32.add $scan $len))
        (when (i32.or (i32.lt_u $cp $DC_WASM_START) (i32.ge_u $cp $DC_WASM_FB_START))
          (return 0))
        (set $sub (i32.sub $cp $DC_WASM_START))
        (block $step
          (when
            (i32.or
              (i32.le_u $sub 0x01)
              (i32.or
                (i32.and (i32.ge_u $sub 0x1A) (i32.le_u $sub 0x1B))
                (i32.and (i32.ge_u $sub 0x45) (i32.le_u $sub 0xC4))))
            (br $step))
          (when (i32.and (i32.ge_u $sub 0x41) (i32.le_u $sub 0x44))
            (when (i32.ge_u $scan $body_end) (return 0))
            (drop (call $raw_decode_at $scan $body_end))
            (when (i32.ne $dec_cp $DC_BEGIN_NUMBER) (return 0))
            (set $scan (call $skip_wasm_balanced $scan $body_end $DC_BEGIN_NUMBER $DC_END_NUMBER))
            (br $step))
          (when (i32.and (i32.ge_u $sub 0x28) (i32.le_u $sub 0x35))
            (set $k 0)
            (block $imm_done
              (loop $imm_lp
                (br_if $imm_done (i32.ge_u $k 3))
                (when (i32.ge_u $scan $body_end) (return 0))
                (drop (call $raw_decode_at $scan $body_end))
                (when (i32.ne $dec_cp $DC_BEGIN_NUMBER) (return 0))
                (set $scan (call $skip_wasm_balanced $scan $body_end $DC_BEGIN_NUMBER $DC_END_NUMBER))
                (set $k (i32.add $k 1))
                (br $imm_lp)))
            (br $step))
          (return 0))
        (br $sc_lp)))

    (call $buf_push_bytes (i32.load offset=16 $fb) $tail_ptr (i32.sub $body_end $tail_ptr))
    (i32.const 1))

  ;; Emits either an inlined pure tail or `(call $fn_idx)` for `($op_cp, $ctx_len, $c0, $c1, $c2)`.
  (func $emit_builtin_op
    (param $fb i32) (param $arity i32) (param $op_cp i32) (param $ctx_len i32) (param $c0 i32) (param $c1 i32) (param $c2 i32)
    (local $matched   i32)
    (local $match_idx i32)
    (set $matched (call $find_builtin_entry $op_cp $ctx_len $c0 $c1 $c2))
    (set $match_idx $ret_aux)
    (when (call $try_inline_builtin $fb $matched $arity) (return))
    (call $fb_op_u32 $fb 0x10 (call $link_builtin_matched $matched $match_idx)))

  (func $emit_scalar_op (param $fb i32) (param $arity i32) (param $op_cp i32) (param $ty_id i32)
    (call $emit_builtin_op $fb $arity $op_cp 1 (call $ty_scalar_fmt_cp $ty_id) 0 0))

  (func $ensure_alloc_helper (result i32)
    (local $p   i32)
    (local $idx i32)
    (when (i32.gt_u $alloc_fn_plus1 0)
      (return (i32.sub $alloc_fn_plus1 1)))
    (drop (call $link_builtin $DC_STAGEL_BUF_NEW 2 $DC_STAGEL_BUF_TYPE $FMT_U8 0))
    (set $p (call $make_ascii 6 0x636F6C6C615F 0)) ;; "_alloc"
    (set $idx (call $find_routine_by_name $p 6))
    (when (i32.lt_s $idx 0) (unreachable))
    (global.set $alloc_fn_plus1 (i32.add $idx 1))
    (get $idx))

  ;; ---------------------------------------------------------------------------
  ;; 4. Canonical Type Table & Composite Layout Engine
  ;; ---------------------------------------------------------------------------

  ;; TypeEntry (32 bytes):
  ;; +0 kind, +4 arg0 (inner_ty / arity / struct_idx), +8 arg1 (slots_ptr),
  ;; +12 layout_state (0/1/2), +16 total_bytes, +20 layout_ptr, +24 eq_fn_plus1
  (func $init_types
    (local $i i32)
    (local $t i32)
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i 8))
        (set $t (call $alloc 32))
        (i32.store $t $i)
        (call $buf_push_i32 $type_table $t)
        (set $i (i32.add $i 1))
        (br $lp)))
    ;; Pre-intern `$TY_TUP0` (`8`) = 0-ary tuple
    (set $t (call $alloc 32))
    (i32.store $t $TK_TUPLE)
    (i32.store offset=4 $t 0)
    (i32.store offset=12 $t 2)
    (i32.store offset=16 $t 0)
    (call $buf_push_i32 $type_table $t))

  ;; Returns WASM value type Format Dc (`$FMT_I32`, `$FMT_I64`, or `0` for Void/Never).
  (func $ty_wasm_vt (param $ty_id i32) (result i32)
    (when (i32.or (i32.eq $ty_id $TY_VOID) (i32.eq $ty_id $TY_NEVER))
      (return 0))
    (when (i32.or (i32.eq $ty_id $TY_I64) (i32.eq $ty_id $TY_U64))
      (return $FMT_I64))
    (i32.const $FMT_I32))

  (func $ty_stride (param $ty_id i32) (result i32)
    (when (i32.or (i32.eq $ty_id $TY_VOID) (i32.eq $ty_id $TY_NEVER))
      (unreachable))
    (when (i32.or (i32.eq $ty_id $TY_B) (i32.eq $ty_id $TY_U8))
      (return 1))
    (when (i32.or (i32.eq $ty_id $TY_I64) (i32.eq $ty_id $TY_U64))
      (return 8))
    (i32.const 4))

  (func $ty_align_exp (param $ty_id i32) (result i32)
    (local $s i32)
    (set $s (call $ty_stride $ty_id))
    (when (i32.eq $s 1) (return 0))
    (when (i32.eq $s 8) (return 3))
    (i32.const 2))

  (func $ty_is_scalar (param $ty_id i32) (result i32)
    (i32.and (i32.ge_u $ty_id $TY_B) (i32.le_u $ty_id $TY_U64)))

  (func $ty_scalar_fmt_cp (param $ty_id i32) (result i32)
    (when (i32.eq $ty_id $TY_B) (return $FMT_BOOL))
    (when (i32.eq $ty_id $TY_U8) (return $FMT_U8))
    (when (i32.eq $ty_id $TY_I32) (return $FMT_I32))
    (when (i32.eq $ty_id $TY_U32) (return $FMT_U32))
    (when (i32.eq $ty_id $TY_I64) (return $FMT_I64))
    (when (i32.eq $ty_id $TY_U64) (return $FMT_U64))
    (unreachable))

  (func $intern_unary_ty (param $kind i32) (param $inner_ty i32) (result i32)
    (local $cnt i32)
    (local $i   i32)
    (local $t   i32)
    (when
      (i32.and
        (i32.ne $kind $TK_STRUCT)
        (i32.or (i32.eq $inner_ty $TY_VOID) (i32.eq $inner_ty $TY_NEVER)))
      (unreachable))
    (set $cnt (call $buf_count_i32 $type_table))
    (set $i 8)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $t (call $buf_get_i32 $type_table $i))
        (when (i32.and (i32.eq (i32.load $t) $kind) (i32.eq (i32.load offset=4 $t) $inner_ty))
          (return $i))
        (set $i (i32.add $i 1))
        (br $lp)))
    (set $t (call $alloc 32))
    (i32.store $t $kind)
    (i32.store offset=4 $t $inner_ty)
    (call $buf_push_i32 $type_table $t)
    (get $cnt))

  (func $intern_tuple_ty (param $arity i32) (param $slots_ptr i32) (result i32)
    (local $cnt i32)
    (local $i   i32)
    (local $j   i32)
    (local $t   i32)
    (local $ok  i32)
    (local $ep  i32)
    (local $sty i32)
    (when (i32.eqz $arity) (return $TY_TUP0))
    (set $j 0)
    (block $vdone
      (loop $vlp
        (br_if $vdone (i32.ge_u $j $arity))
        (set $sty (i32.load (i32.add $slots_ptr (i32.shl $j 2))))
        (when (i32.or (i32.eq $sty $TY_VOID) (i32.eq $sty $TY_NEVER)) (unreachable))
        (set $j (i32.add $j 1))
        (br $vlp)))
    (set $cnt (call $buf_count_i32 $type_table))
    (set $i 8)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $t (call $buf_get_i32 $type_table $i))
        (when (i32.and (i32.eq (i32.load $t) $TK_TUPLE) (i32.eq (i32.load offset=4 $t) $arity))
          (set $ep (i32.load offset=8 $t))
          (set $ok 1)
          (set $j 0)
          (block $cdone
            (loop $clp
              (br_if $cdone (i32.ge_u $j $arity))
              (when
                (i32.ne
                  (i32.load (i32.add $ep (i32.shl $j 2)))
                  (i32.load (i32.add $slots_ptr (i32.shl $j 2))))
                (set $ok 0)
                (br $cdone))
              (set $j (i32.add $j 1))
              (br $clp)))
          (when $ok (return $i)))
        (set $i (i32.add $i 1))
        (br $lp)))
    (set $t (call $alloc 32))
    (i32.store $t $TK_TUPLE)
    (i32.store offset=4 $t $arity)
    (i32.store offset=8 $t $slots_ptr)
    (call $buf_push_i32 $type_table $t)
    (get $cnt))

  (func $intern_struct_ty (param $sdef_idx i32) (result i32)
    (call $intern_unary_ty $TK_STRUCT $sdef_idx))

  ;; ---------------------------------------------------------------------------
  ;; 5. StageL 3 Token-Dc Recursive-Descent Parser
  ;; ---------------------------------------------------------------------------

  ;; Parses `Dc 270 .. Dc 271` StageL identifier, returning `ptr` and setting `$ret_len`.
  (func $parse_ident (result i32)
    (local $b  i32)
    (local $cp i32)
    (call $expect_tok $DC_BEGIN_IDENTIFIER)
    (set $b (call $buf_new 16))
    (block $done
      (loop $lp
        (set $cp (call $raw_next))
        (br_if $done (i32.eq $cp $DC_END_IDENTIFIER))
        (when (i32.eq $cp $DC_ESCAPE)
          (set $cp (call $raw_next)))
        (when (i32.or (i32.lt_s $cp 0) (i32.ge_u $cp 0x110000)) (unreachable))
        (call $emit_cp $b $cp)
        (br $lp)))
    (when (i32.eqz (i32.load offset=4 $b)) (unreachable))
    (global.set $ret_len (i32.load offset=4 $b))
    (i32.load $b))

  ;; Parses `f229` FilePath into `'/'`-joined namespace key, returning `ptr` and `$ret_len`.
  (func $parse_filepath (result i32)
    (local $b   i32)
    (local $cp  i32)
    (local $seg i32)
    (call $expect_tok $DC_BEGIN_FORMAT_REPR)
    (when (i32.ne (call $raw_next) $FMT_FILEPATH) (unreachable))
    (when (i32.ne (call $raw_next) $DC_BEGIN_LIST) (unreachable))
    (when (i32.ne (call $raw_next) $DC_TYPE_INTERSECTION) (unreachable))
    (when (i32.ne (call $raw_next) $FMT_FILENAME) (unreachable))
    (when (i32.ne (call $raw_next) $DC_FILE_PATH_DELIM) (unreachable))
    (set $b (call $buf_new 32))
    (set $seg 0)
    (block $done
      (loop $lp
        (when (i32.gt_u $seg 0)
          (call $buf_push_u8 $b 47)) ;; '/'
        (when (i32.ne (call $raw_next) $DC_BEGIN_FORMAT_REPR) (unreachable))
        (when (i32.ne (call $raw_next) $FMT_FILENAME) (unreachable))
        (when (i32.ne (call $raw_next) $DC_BEGIN_LITERAL) (unreachable))
        (when (i32.ne (call $raw_next) $DC_BEGIN_TYPE_DEF) (unreachable))
        (when (i32.ne (call $raw_next) $DC_TYPE_STRING) (unreachable))
        (when (i32.ne (call $raw_next) $DC_END_TYPE_DEF) (unreachable))
        (block $sdone
          (loop $slp
            (set $cp (call $raw_next))
            (br_if $sdone (i32.eq $cp $DC_END_LITERAL))
            (when (i32.eq $cp $DC_ESCAPE)
              (set $cp (call $raw_next)))
            (when (i32.or (i32.lt_s $cp 0) (i32.ge_u $cp 0x110000)) (unreachable))
            (call $emit_cp $b $cp)
            (br $slp)))
        (when (i32.ne (call $raw_next) $DC_END_FORMAT_REPR) (unreachable))
        (set $seg (i32.add $seg 1))
        (set $cp (call $raw_next))
        (br_if $done (i32.eq $cp $DC_END_LIST))
        (when (i32.ne $cp $DC_FILE_PATH_DELIM) (unreachable))
        (br $lp)))
    (when (i32.ne (call $raw_next) $DC_END_FORMAT_REPR) (unreachable))
    (global.set $ret_len (i32.load offset=4 $b))
    (i32.load $b))

  ;; Parses an `atom_dc` (honoring `Dc 255` escape), returning `raw_ptr` and setting `$ret_len`.
  (func $parse_atom_dc (result i32)
    (local $cp i32)
    (set $cp (call $tok_next))
    (when (i32.eq $cp $DC_ESCAPE)
      (drop (call $raw_next)))
    (global.set $ret_len $dec_raw_len)
    (global.get $dec_raw_ptr))

  ;; Parses optional `atom_dc` (`Dc 316` -> `ptr = 0, $ret_len = 0`).
  (func $parse_opt_atom_dc (result i32)
    (if (i32.eq (call $tok_peek) $DC_OPTIONAL_ABSENT)
      (then
        (drop (call $tok_next))
        (global.set $ret_len 0)
        (return 0))
      (else
        (return (call $parse_atom_dc))))
    (unreachable))

  ;; Parses `SymbolTarget`: sets `$ret_aux = is_dc (0/1)`, `$ret_len`, returns `ptr`.
  (func $parse_sym_target (result i32)
    (local $p i32)
    (if (i32.eq (call $tok_peek) $DC_BEGIN_IDENTIFIER)
      (then
        (set $p (call $parse_ident))
        (global.set $ret_aux 0)
        (return $p))
      (else
        (set $p (call $parse_atom_dc))
        (global.set $ret_aux 1)
        (return $p)))
    (unreachable))

  ;; Parses unsigned `u32` Dc number `Dc 6 f199 <base64>+ Dc 7` from token stream.
  (func $read_u32_dc_num (result i32)
    (local $v i32)
    (call $skip_comments)
    (set $v (call $read_wasm_u32_at $cur_ptr $cur_end))
    (global.set $cur_ptr $ret_aux)
    (get $v))

  ;; Parses raw `SyntaxType` node (16 bytes: `+0 kind`, `+4 a`, `+8 b`).
  ;; Primitive types return a singleton `SyntaxType` whose `+0 kind` is `0..=7`.
  (func $parse_raw_type (result i32)
    (local $cp    i32)
    (local $node  i32)
    (local $elems i32)
    (local $p     i32)
    (set $cp (call $tok_next))
    (set $node (call $alloc 16))
    (when (i32.eq $cp $FMT_VOID)  (i32.store $node $TY_VOID)  (return $node))
    (when (i32.eq $cp $FMT_NEVER) (i32.store $node $TY_NEVER) (return $node))
    (when (i32.eq $cp $FMT_BOOL)  (i32.store $node $TY_B)     (return $node))
    (when (i32.eq $cp $FMT_U8)    (i32.store $node $TY_U8)    (return $node))
    (when (i32.eq $cp $FMT_I32)   (i32.store $node $TY_I32)   (return $node))
    (when (i32.eq $cp $FMT_U32)   (i32.store $node $TY_U32)   (return $node))
    (when (i32.eq $cp $FMT_I64)   (i32.store $node $TY_I64)   (return $node))
    (when (i32.eq $cp $FMT_U64)   (i32.store $node $TY_U64)   (return $node))
    (when (i32.eq $cp $DC_STAGEL_LIST_TYPE)
      (i32.store $node $TK_LIST)
      (i32.store offset=4 $node (call $parse_raw_type))
      (return $node))
    (when (i32.eq $cp $DC_STAGEL_BUF_TYPE)
      (i32.store $node $TK_BUF)
      (i32.store offset=4 $node (call $parse_raw_type))
      (return $node))
    (when (i32.eq $cp $DC_TYPE_TUPLE)
      (call $expect_tok $DC_BEGIN_LIST)
      (call $expect_tok $DC_BEGIN_TYPE_DEF)
      (call $expect_tok $DC_TYPE_ANY)
      (call $expect_tok $DC_END_TYPE_DEF)
      (set $elems (call $buf_new 16))
      (when (i32.ne (call $tok_peek) $DC_END_LIST)
        (block $tdone
          (loop $tlp
            (call $buf_push_i32 $elems (call $parse_raw_type))
            (br_if $tdone (i32.eq (call $tok_peek) $DC_END_LIST))
            (call $expect_tok $DC_LIST_SEP)
            (br $tlp))))
      (call $expect_tok $DC_END_LIST)
      (i32.store $node $TK_TUPLE)
      (i32.store offset=4 $node (call $buf_count_i32 $elems))
      (i32.store offset=8 $node (i32.load $elems))
      (return $node))
    (when (i32.eq $cp $DC_TYPE_STRUCT)
      (set $p (call $parse_sym_target))
      (i32.store $node
        (if (result i32) $ret_aux
          (then (i32.const $TK_RAW_SDC))
          (else (i32.const $TK_RAW_SIDENT))))
      (i32.store offset=4 $node $p)
      (i32.store offset=8 $node $ret_len)
      (return $node))
    (unreachable))

  ;; Parses `TypedIntLit` (`Dc 6 <fmt_dc> (Dc 11)? <base64>+ Dc 7`) into `EX_INT` node.
  (func $parse_typed_int (result i32)
    (local $fmt    i32)
    (local $is_neg i32)
    (local $cp     i32)
    (local $mag    i64)
    (local $ndig   i32)
    (local $ty_id  i32)
    (local $val    i64)
    (local $ex     i32)

    (set $fmt (call $raw_next))
    (set $is_neg 0)
    (when (i32.eq (call $raw_peek) $DC_NEGATIVE)
      (drop (call $raw_next))
      (set $is_neg 1))
    (set $mag 0)
    (set $ndig 0)
    (block $done
      (loop $lp
        (set $cp (call $raw_next))
        (br_if $done (i32.eq $cp $DC_END_NUMBER))
        (when (i32.or (i32.lt_u $cp $DC_BASE64_START) (i32.gt_u $cp $DC_BASE64_END))
          (unreachable))
        (when (i64.ne (i64.shr_u $mag 58) 0) (unreachable))
        (set $mag
          (i64.or
            (i64.shl $mag 6)
            (i64.extend_i32_u (i32.sub $cp $DC_BASE64_START))))
        (set $ndig (i32.add $ndig 1))
        (br $lp)))
    (when (i32.eqz $ndig) (unreachable))

    (if (i32.eq $fmt $FMT_U8)
      (then
        (when (i32.or $is_neg (i64.gt_u $mag 255)) (unreachable))
        (set $ty_id $TY_U8)
        (set $val $mag))
      (else
        (if (i32.eq $fmt $FMT_U32)
          (then
            (when (i32.or $is_neg (i64.gt_u $mag 0xFFFFFFFF)) (unreachable))
            (set $ty_id $TY_U32)
            (set $val (i64.extend_i32_s (i32.wrap_i64 $mag))))
          (else
            (if (i32.eq $fmt $FMT_I32)
              (then
                (set $ty_id $TY_I32)
                (if $is_neg
                  (then
                    (when (i64.gt_u $mag 0x80000000) (unreachable))
                    (set $val (i64.sub 0 $mag)))
                  (else
                    (when (i64.gt_u $mag 0x7FFFFFFF) (unreachable))
                    (set $val $mag))))
              (else
                (if (i32.eq $fmt $FMT_U64)
                  (then
                    (when $is_neg (unreachable))
                    (set $ty_id $TY_U64)
                    (set $val $mag))
                  (else
                    (when (i32.ne $fmt $FMT_I64) (unreachable))
                    (set $ty_id $TY_I64)
                    (if $is_neg
                      (then
                        (when (i64.gt_u $mag 0x8000000000000000) (unreachable))
                        (set $val (i64.sub 0 $mag)))
                      (else
                        (when (i64.gt_u $mag 0x7FFFFFFFFFFFFFFF) (unreachable))
                        (set $val $mag)))))))))))

    (set $ex (call $alloc 32))
    (i32.store $ex $EX_INT)
    (i32.store offset=4 $ex $ty_id)
    (i64.store offset=24 $ex $val)
    (get $ex))

  ;; Parses Base64 binary bytes `Dc 203 <u32_len> <b64_quads>* Dc 204` into `EX_BYTES`.
  (func $parse_bytes_lit (result i32)
    (local $blen i32)
    (local $rem  i32)
    (local $out  i32)
    (local $d0   i32)
    (local $d1   i32)
    (local $d2   i32)
    (local $d3   i32)
    (local $ex   i32)

    (set $blen (call $read_u32_dc_num))
    (set $out (call $buf_new (i32.add $blen 8)))
    (set $rem $blen)
    (block $done
      (loop $lp
        (br_if $done (i32.eqz $rem))
        (set $d0 (i32.sub (call $raw_next) $DC_BASE64_START))
        (set $d1 (i32.sub (call $raw_next) $DC_BASE64_START))
        (when (i32.or (i32.gt_u $d0 63) (i32.gt_u $d1 63)) (unreachable))
        (call $buf_push_u8 $out (i32.or (i32.shl $d0 2) (i32.shr_u $d1 4)))
        (if (i32.ge_u $rem 3)
          (then
            (set $d2 (i32.sub (call $raw_next) $DC_BASE64_START))
            (set $d3 (i32.sub (call $raw_next) $DC_BASE64_START))
            (when (i32.or (i32.gt_u $d2 63) (i32.gt_u $d3 63)) (unreachable))
            (call $buf_push_u8 $out (i32.and 255 (i32.or (i32.shl $d1 4) (i32.shr_u $d2 2))))
            (call $buf_push_u8 $out (i32.and 255 (i32.or (i32.shl $d2 6) $d3)))
            (set $rem (i32.sub $rem 3)))
          (else
            (if (i32.eq $rem 2)
              (then
                (set $d2 (i32.sub (call $raw_next) $DC_BASE64_START))
                (when (i32.gt_u $d2 63) (unreachable))
                (when (i32.ne (call $raw_next) $DC_BASE64_PADDING) (unreachable))
                (call $buf_push_u8 $out (i32.and 255 (i32.or (i32.shl $d1 4) (i32.shr_u $d2 2))))
                (set $rem 0))
              (else
                (when (i32.ne (call $raw_next) $DC_BASE64_PADDING) (unreachable))
                (when (i32.ne (call $raw_next) $DC_BASE64_PADDING) (unreachable))
                (set $rem 0)))))
        (br $lp)))
    (when (i32.ne (call $raw_next) $DC_END_ENCAP_BINARY) (unreachable))
    (set $ex (call $alloc 32))
    (i32.store $ex $EX_BYTES)
    (i32.store offset=4 $ex (i32.load $out))
    (i32.store offset=8 $ex $blen)
    (get $ex))

  (func $skip_test_block
    (local $depth i32)
    (local $cp    i32)
    (set $depth 1)
    (block $done
      (loop $lp
        (br_if $done (i32.eqz $depth))
        (set $cp (call $raw_next))
        (if (i32.eq $cp $DC_ESCAPE)
          (then (drop (call $raw_next)))
          (else
            (if (i32.eq $cp $DC_BEGIN_TEST_BLOCK)
              (then (set $depth (i32.add $depth 1)))
              (else
                (when (i32.eq $cp $DC_END_TEST_BLOCK)
                  (set $depth (i32.sub $depth 1)))))))
        (br $lp))))

  ;; RoutineAst (48 bytes):
  ;; +0 is_pub, +4 dc_ptr, +8 dc_len, +12 raw_ret_ty, +16 name_ptr, +20 name_len,
  ;; +24 params_buf (ParamAst*: +0 raw_ty, +4 n_ptr, +8 n_len, +12 canon_ty),
  ;; +28 body_expr, +32 canon_ret_ty, +36 func_idx
  (func $parse_routine_after_open (result i32)
    (local $r      i32)
    (local $bcp    i32)
    (local $params i32)
    (local $p      i32)
    (local $ptr    i32)

    (set $r (call $alloc 48))
    (set $bcp (call $tok_next))
    (if (i32.eq $bcp $DC_BOOLEAN_TRUE)
      (then (i32.store $r 1))
      (else
        (when (i32.ne $bcp $DC_BOOLEAN_FALSE) (unreachable))
        (i32.store $r 0)))
    (set $ptr (call $parse_opt_atom_dc))
    (i32.store offset=4 $r $ptr)
    (i32.store offset=8 $r $ret_len)
    (i32.store offset=12 $r (call $parse_raw_type))
    (set $ptr (call $parse_ident))
    (i32.store offset=16 $r $ptr)
    (i32.store offset=20 $r $ret_len)

    (set $params (call $buf_new 16))
    (block $pdone
      (loop $plp
        (br_if $pdone (i32.ne (call $tok_peek) $DC_ROUTINE_PARAM))
        (drop (call $tok_next))
        (set $p (call $alloc 16))
        (i32.store $p (call $parse_raw_type))
        (set $ptr (call $parse_ident))
        (i32.store offset=4 $p $ptr)
        (i32.store offset=8 $p $ret_len)
        (call $buf_push_i32 $params $p)
        (br $plp)))
    (i32.store offset=24 $r $params)

    (block $tdone
      (loop $tlp
        (br_if $tdone (i32.ne (call $tok_peek) $DC_BEGIN_TEST_BLOCK))
        (drop (call $tok_next))
        (call $skip_test_block)
        (br $tlp)))

    (i32.store offset=28 $r (call $parse_expr))
    (call $expect_tok $DC_END_ROUTINE_DECL)
    (get $r))

  (func $is_unary_op_cp (param $cp i32) (result i32)
    (i32.or
      (i32.or
        (i32.or (i32.eq $cp $DC_NEG_OP) (i32.eq $cp $DC_BIT_NOT_OP))
        (i32.or (i32.eq $cp $DC_LOGICAL_NOT_OP) (i32.eq $cp $DC_SEQ_LEN_OP)))
      (i32.or
        (i32.and (i32.ge_u $cp $DC_CLZ_OP) (i32.le_u $cp $DC_POPCNT_OP))
        (i32.or
          (i32.eq $cp $DC_STAGEL_BUF_FREEZE)
          (i32.eq $cp $DC_REQUEST_HOST_CAPABILITY_OP)))))

  (func $is_binary_op_cp (param $cp i32) (result i32)
    (i32.or
      (i32.or
        (i32.and (i32.ge_u $cp $DC_ADD_OP) (i32.le_u $cp $DC_SUB_OP))
        (i32.and (i32.ge_u $cp $DC_MUL_OP) (i32.le_u $cp $DC_REM_OP)))
      (i32.or
        (i32.or
          (i32.and (i32.ge_u $cp $DC_WRAP_ADD_OP) (i32.le_u $cp $DC_BIT_XOR_OP))
          (i32.and (i32.ge_u $cp $DC_SHL_OP) (i32.le_u $cp $DC_ROTR_OP)))
        (i32.or
          (i32.and (i32.ge_u $cp $DC_LOGICAL_AND_OP) (i32.le_u $cp $DC_GE_OP))
          (i32.or
            (i32.or (i32.eq $cp $DC_SEQ_GET_OP) (i32.eq $cp $DC_SEQ_CONCAT_OP))
            (i32.and (i32.ge_u $cp $DC_STAGEL_BUF_FILL) (i32.le_u $cp $DC_STAGEL_BUF_EXTEND)))))))

  (func $parse_expr (result i32)
    (local $cp    i32)
    (local $ex    i32)
    (local $elems i32)
    (local $arity i32)
    (local $i     i32)
    (local $p     i32)

    (set $cp (call $tok_next))
    (set $ex (call $alloc 32))

    (when (i32.eq $cp $DC_BOOLEAN_TRUE)
      (i32.store $ex $EX_BOOL)
      (i32.store offset=4 $ex 1)
      (return $ex))
    (when (i32.eq $cp $DC_BOOLEAN_FALSE)
      (i32.store $ex $EX_BOOL)
      (i32.store offset=4 $ex 0)
      (return $ex))
    (when (i32.eq $cp $DC_BEGIN_NUMBER)
      (return (call $parse_typed_int)))
    (when (i32.eq $cp $DC_START_ENCAP_BINARY)
      (return (call $parse_bytes_lit)))
    (when (i32.eq $cp $DC_BEGIN_STAGEL_LIST)
      (i32.store $ex $EX_LIST)
      (i32.store offset=4 $ex (call $parse_raw_type))
      (set $elems (call $buf_new 16))
      (block $ldone
        (loop $llp
          (br_if $ldone (i32.eq (call $tok_peek) $DC_END_STAGEL_LIST))
          (call $buf_push_i32 $elems (call $parse_expr))
          (br $llp)))
      (call $expect_tok $DC_END_STAGEL_LIST)
      (i32.store offset=8 $ex $elems)
      (return $ex))
    (when (i32.eq $cp $DC_BEGIN_TUPLE)
      (set $arity (call $read_u32_dc_num))
      (set $elems (call $buf_new 16))
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $arity))
          (when (i32.gt_u $i 0) (call $expect_tok $DC_LIST_SEP))
          (call $buf_push_i32 $elems (call $parse_expr))
          (set $i (i32.add $i 1))
          (br $tlp)))
      (call $expect_tok $DC_END_TUPLE)
      (i32.store $ex $EX_TUPLE_NEW)
      (i32.store offset=4 $ex $elems)
      (return $ex))
    (when (i32.eq $cp $DC_BEGIN_POS_STRUCT_VAL)
      (set $p (call $parse_sym_target))
      (i32.store $ex $EX_STRUCT_NEW)
      (i32.store offset=4 $ex $ret_aux)
      (i32.store offset=8 $ex $p)
      (i32.store offset=12 $ex $ret_len)
      (set $elems (call $buf_new 16))
      (block $sdone
        (loop $slp
          (br_if $sdone (i32.eq (call $tok_peek) $DC_END_POS_STRUCT_VAL))
          (call $buf_push_i32 $elems (call $parse_expr))
          (br $slp)))
      (call $expect_tok $DC_END_POS_STRUCT_VAL)
      (when (i32.eqz (call $buf_count_i32 $elems)) (unreachable))
      (i32.store offset=16 $ex $elems)
      (return $ex))
    (when (i32.eq $cp $DC_GET_POS_FIELD_OP)
      (i32.store $ex $EX_FIELD)
      (i32.store offset=4 $ex (call $parse_expr))
      (i32.store offset=8 $ex (call $read_u32_dc_num))
      (return $ex))
    (when (i32.eq $cp $DC_WITH_POS_FIELD_OP)
      (i32.store $ex $EX_WITH)
      (i32.store offset=4 $ex (call $parse_expr))
      (i32.store offset=8 $ex (call $read_u32_dc_num))
      (i32.store offset=12 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_STAGEL_BUF_NEW)
      (i32.store $ex $EX_BUF_NEW)
      (i32.store offset=4 $ex (call $parse_raw_type))
      (i32.store offset=8 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_TYPED_VALUE_REF)
      (i32.store $ex $EX_VALUE_REF)
      (if (i32.eq (call $tok_peek) $DC_OPTIONAL_ABSENT)
        (then
          (drop (call $tok_next))
          (i32.store offset=4 $ex 0))
        (else
          (i32.store offset=4 $ex (call $parse_raw_type))))
      (set $p (call $parse_sym_target))
      (i32.store offset=8 $ex $ret_aux)
      (i32.store offset=12 $ex $p)
      (i32.store offset=16 $ex $ret_len)
      (return $ex))
    (when (i32.eq $cp $DC_DECLARE_LET)
      (i32.store $ex $EX_LET)
      (i32.store offset=4 $ex (call $parse_raw_type))
      (set $p (call $parse_ident))
      (i32.store offset=8 $ex $p)
      (i32.store offset=12 $ex $ret_len)
      (if (i32.eq (call $tok_peek) $DC_OPTIONAL_ABSENT)
        (then
          (drop (call $tok_next))
          (i32.store offset=16 $ex 0))
        (else
          (i32.store offset=16 $ex (call $parse_expr))))
      (return $ex))
    (when (i32.eq $cp $DC_DECLARE_CONST)
      (i32.store $ex $EX_CONST)
      (drop (call $parse_opt_atom_dc))
      (i32.store offset=4 $ex (call $parse_raw_type))
      (set $p (call $parse_ident))
      (i32.store offset=8 $ex $p)
      (i32.store offset=12 $ex $ret_len)
      (i32.store offset=16 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_ASSIGN_SET)
      (i32.store $ex $EX_SET)
      (if (i32.eq (call $tok_peek) $DC_OPTIONAL_ABSENT)
        (then
          (drop (call $tok_next))
          (i32.store offset=4 $ex 0))
        (else
          (i32.store offset=4 $ex (call $parse_raw_type))))
      (set $p (call $parse_ident))
      (i32.store offset=8 $ex $p)
      (i32.store offset=12 $ex $ret_len)
      (i32.store offset=16 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_BEGIN_ROUTINE_DECL)
      (global.set $uses_shadow 1)
      (i32.store $ex $EX_ROUTINE)
      (i32.store offset=4 $ex (call $parse_routine_after_open))
      (return $ex))
    (when (i32.eq $cp $DC_BEGIN_ROUTINE_CALL)
      (i32.store $ex $EX_CALL)
      (set $p (call $parse_sym_target))
      (i32.store offset=4 $ex $ret_aux)
      (i32.store offset=8 $ex $p)
      (i32.store offset=12 $ex $ret_len)
      (set $elems (call $buf_new 16))
      (block $cdone
        (loop $clp
          (br_if $cdone (i32.eq (call $tok_peek) $DC_END_ROUTINE_CALL))
          (call $buf_push_i32 $elems (call $parse_expr))
          (br $clp)))
      (call $expect_tok $DC_END_ROUTINE_CALL)
      (i32.store offset=16 $ex $elems)
      (return $ex))
    (when (i32.eq $cp $DC_BEGIN_CODE_BLOCK)
      (i32.store $ex $EX_BLOCK)
      (set $elems (call $buf_new 16))
      (block $bdone
        (loop $blp
          (br_if $bdone (i32.eq (call $tok_peek) $DC_END_CODE_BLOCK))
          (call $buf_push_i32 $elems (call $parse_expr))
          (br $blp)))
      (call $expect_tok $DC_END_CODE_BLOCK)
      (i32.store offset=4 $ex $elems)
      (return $ex))
    (when (i32.eq $cp $DC_IF_CONDITIONAL)
      (i32.store $ex $EX_IF)
      (i32.store offset=4 $ex (call $parse_expr))
      (i32.store offset=8 $ex (call $parse_expr))
      (i32.store offset=12 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_WHILE_LOOP)
      (i32.store $ex $EX_WHILE)
      (i32.store offset=4 $ex (call $parse_expr))
      (i32.store offset=8 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_FOR_EACH)
      (i32.store $ex $EX_FOREACH)
      (i32.store offset=4 $ex (call $parse_raw_type))
      (set $p (call $parse_ident))
      (i32.store offset=8 $ex $p)
      (i32.store offset=12 $ex $ret_len)
      (i32.store offset=16 $ex (call $parse_expr))
      (i32.store offset=20 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_BREAK_LOOP)
      (i32.store $ex $EX_BREAK)
      (return $ex))
    (when (i32.eq $cp $DC_CONTINUE_LOOP)
      (i32.store $ex $EX_CONTINUE)
      (return $ex))
    (when (i32.eq $cp $DC_RETURN_ROUTINE)
      (i32.store $ex $EX_RETURN)
      (if (i32.eq (call $tok_peek) $DC_OPTIONAL_ABSENT)
        (then
          (drop (call $tok_next))
          (i32.store offset=4 $ex 0))
        (else
          (i32.store offset=4 $ex (call $parse_expr))))
      (return $ex))
    (when (i32.eq $cp $DC_FAULT_TRAP)
      (i32.store $ex $EX_FAULT)
      (return $ex))
    (when (i32.eq $cp $DC_NO_OP)
      (i32.store $ex $EX_NOP)
      (return $ex))
    (when (call $is_unary_op_cp $cp)
      (i32.store $ex $EX_UNARY)
      (i32.store offset=4 $ex $cp)
      (i32.store offset=8 $ex (call $parse_expr))
      (return $ex))
    (when (call $is_binary_op_cp $cp)
      (i32.store $ex $EX_BINARY)
      (i32.store offset=4 $ex $cp)
      (i32.store offset=8 $ex (call $parse_expr))
      (i32.store offset=12 $ex (call $parse_expr))
      (return $ex))
    (when (i32.or (i32.eq $cp $DC_SEQ_SLICE_OP) (i32.eq $cp $DC_STAGEL_BUF_SET))
      (i32.store $ex $EX_TERNARY)
      (i32.store offset=4 $ex $cp)
      (i32.store offset=8 $ex (call $parse_expr))
      (i32.store offset=12 $ex (call $parse_expr))
      (i32.store offset=16 $ex (call $parse_expr))
      (return $ex))
    (when (i32.eq $cp $DC_CONVERT_OP)
      (i32.store $ex $EX_CONV)
      (i32.store offset=4 $ex (call $parse_expr))
      (i32.store offset=8 $ex (call $parse_raw_type))
      (return $ex))
    (when (i32.eq $cp $DC_BITCAST_OP)
      (i32.store $ex $EX_BITCAST)
      (i32.store offset=4 $ex (call $parse_expr))
      (i32.store offset=8 $ex (call $parse_raw_type))
      (return $ex))
    (unreachable))

  ;; ModAst (32 bytes):
  ;; +0 ns_ptr, +4 ns_len, +8 uses_buf (UseItem*: +0 ns_ptr, +4 ns_len, +8 is_dc, +12 t_ptr, +16 t_len, +20 alias_ptr, +24 alias_len),
  ;; +12 structs_buf (StructDef* indices), +16 vals_buf (TopVal*), +20 routs_buf (RoutineAst*)
  ;; StructDef (48 bytes):
  ;; +0 mod_idx, +4 is_pub, +8 dc_ptr, +12 dc_len, +16 n_ptr, +20 n_len, +24 raw_fields_buf, +28 canon_fields_ptr, +32 arity, +36 visit_state
  ;; TopVal (40 bytes):
  ;; +0 is_pub, +4 dc_ptr, +8 dc_len, +12 raw_ty, +16 n_ptr, +20 n_len, +24 val_expr, +28 canon_ty, +32 glob_idx
  (func $parse_module (param $mod_idx i32) (result i32)
    (local $m        i32)
    (local $cp       i32)
    (local $p        i32)
    (local $ns_p     i32)
    (local $ns_l     i32)
    (local $u_ns_p   i32)
    (local $u_ns_l   i32)
    (local $uitem    i32)
    (local $sdef     i32)
    (local $s_idx    i32)
    (local $rfields  i32)
    (local $tv       i32)
    (local $uses     i32)
    (local $structs  i32)
    (local $vals     i32)
    (local $routs    i32)

    (call $expect_tok $DC_BEGIN_MODULE)
    (set $m (call $alloc 32))
    (set $uses (call $buf_new 16))
    (set $structs (call $buf_new 16))
    (set $vals (call $buf_new 16))
    (set $routs (call $buf_new 16))
    (i32.store offset=8 $m $uses)
    (i32.store offset=12 $m $structs)
    (i32.store offset=16 $m $vals)
    (i32.store offset=20 $m $routs)

    (set $cp (call $tok_next))
    (if (i32.eq $cp $DC_NAMESPACE_DECL)
      (then
        (set $ns_p (call $parse_filepath))
        (set $ns_l $ret_len)
        (i32.store $m $ns_p)
        (i32.store offset=4 $m $ns_l))
      (else
        (when (i32.ne $cp $DC_OPTIONAL_ABSENT) (unreachable))))

    ;; `use` imports
    (block $udone
      (loop $ulp
        (br_if $udone (i32.ne (call $tok_peek) $DC_BEGIN_USE_IMPORT))
        (drop (call $tok_next))
        (set $u_ns_p (call $parse_filepath))
        (set $u_ns_l $ret_len)
        (block $idone
          (loop $ilp
            (br_if $idone (i32.eq (call $tok_peek) $DC_END_USE_IMPORT))
            (call $expect_tok $DC_USE_IMPORT_ITEM)
            (set $uitem (call $alloc 32))
            (i32.store $uitem $u_ns_p)
            (i32.store offset=4 $uitem $u_ns_l)
            (set $p (call $parse_sym_target))
            (i32.store offset=8 $uitem $ret_aux)
            (i32.store offset=12 $uitem $p)
            (i32.store offset=16 $uitem $ret_len)
            (if (i32.eq (call $tok_peek) $DC_OPTIONAL_ABSENT)
              (then
                (drop (call $tok_next))
                (when $ret_aux (unreachable)) ;; Dc import without alias is invalid
                (i32.store offset=20 $uitem $p)
                (i32.store offset=24 $uitem (i32.load offset=16 $uitem)))
              (else
                (set $p (call $parse_ident))
                (i32.store offset=20 $uitem $p)
                (i32.store offset=24 $uitem $ret_len)))
            (call $buf_push_i32 $uses $uitem)
            (br $ilp)))
        (call $expect_tok $DC_END_USE_IMPORT)
        (br $ulp)))

    ;; Top-level declarations
    (block $ddone
      (loop $dlp
        (set $cp (call $tok_next))
        (br_if $ddone (i32.eq $cp $DC_END_MODULE))
        (if (i32.eq $cp $DC_BEGIN_POS_STRUCT_DECL)
          (then
            (set $sdef (call $alloc 48))
            (i32.store $sdef $mod_idx)
            (set $cp (call $tok_next))
            (if (i32.eq $cp $DC_BOOLEAN_TRUE)
              (then (i32.store offset=4 $sdef 1))
              (else
                (when (i32.ne $cp $DC_BOOLEAN_FALSE) (unreachable))
                (i32.store offset=4 $sdef 0)))
            (set $p (call $parse_opt_atom_dc))
            (i32.store offset=8 $sdef $p)
            (i32.store offset=12 $sdef $ret_len)
            (set $p (call $parse_ident))
            (i32.store offset=16 $sdef $p)
            (i32.store offset=20 $sdef $ret_len)
            (set $rfields (call $buf_new 16))
            (block $sfdone
              (loop $sflp
                (br_if $sfdone (i32.eq (call $tok_peek) $DC_END_POS_STRUCT_DECL))
                (call $buf_push_i32 $rfields (call $parse_raw_type))
                (br $sflp)))
            (call $expect_tok $DC_END_POS_STRUCT_DECL)
            (when (i32.eqz (call $buf_count_i32 $rfields)) (unreachable))
            (i32.store offset=24 $sdef $rfields)
            (set $s_idx (call $buf_count_i32 $all_structs))
            (call $buf_push_i32 $all_structs $sdef)
            (call $buf_push_i32 $structs $s_idx))
          (else
            (if (i32.or (i32.eq $cp $DC_DECLARE_PUB) (i32.eq $cp $DC_DECLARE_CONST))
              (then
                (global.set $has_top_vals 1)
                (set $tv (call $alloc 40))
                (i32.store $tv (i32.eq $cp $DC_DECLARE_PUB))
                (set $p (call $parse_opt_atom_dc))
                (i32.store offset=4 $tv $p)
                (i32.store offset=8 $tv $ret_len)
                (i32.store offset=12 $tv (call $parse_raw_type))
                (set $p (call $parse_ident))
                (i32.store offset=16 $tv $p)
                (i32.store offset=20 $tv $ret_len)
                (i32.store offset=24 $tv (call $parse_expr))
                (call $buf_push_i32 $vals $tv))
              (else
                (when (i32.ne $cp $DC_BEGIN_ROUTINE_DECL) (unreachable))
                (call $buf_push_i32 $routs (call $parse_routine_after_open))))))
        (br $dlp)))

    (get $m))


  ;; ---------------------------------------------------------------------------
  ;; 6. Symbol Resolution, Type Canonicalization & Layout Validation
  ;; ---------------------------------------------------------------------------

  (func $buf_push_sanitized (param $b i32) (param $p i32) (param $l i32)
    (local $i  i32)
    (local $ch i32)
    (local $ok i32)
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $l))
        (set $ch (i32.load8_u (i32.add $p $i)))
        (set $ok
          (i32.or
            (i32.or
              (i32.and (i32.ge_u $ch 97) (i32.le_u $ch 122))
              (i32.and (i32.ge_u $ch 65) (i32.le_u $ch 90)))
            (i32.or
              (i32.and (i32.ge_u $ch 48) (i32.le_u $ch 57))
              (i32.eq $ch 95))))
        (call $buf_push_u8 $b (if (result i32) $ok (then (get $ch)) (else (i32.const 95))))
        (set $i (i32.add $i 1))
        (br $lp))))

  (func $buf_push_cstr4 (param $b i32) (param $len i32) (param $packed i32)
    (local $i i32)
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $len))
        (call $buf_push_u8 $b (i32.and 255 (i32.shr_u $packed (i32.shl $i 3))))
        (set $i (i32.add $i 1))
        (br $lp))))

  (func $buf_push_type_slug (param $b i32) (param $ty_id i32)
    (local $t     i32)
    (local $kind  i32)
    (local $arity i32)
    (local $sp    i32)
    (local $i     i32)
    (local $sdef  i32)
    (local $m     i32)

    (when (i32.eq $ty_id $TY_B)     (call $buf_push_u8 $b 98)           (return)) ;; "b"
    (when (i32.eq $ty_id $TY_U8)    (call $buf_push_cstr4 $b 2 0x3875)  (return)) ;; "u8"
    (when (i32.eq $ty_id $TY_I32)   (call $buf_push_cstr4 $b 3 0x323369)(return)) ;; "i32"
    (when (i32.eq $ty_id $TY_U32)   (call $buf_push_cstr4 $b 3 0x323375)(return)) ;; "u32"
    (when (i32.eq $ty_id $TY_I64)   (call $buf_push_cstr4 $b 3 0x343669)(return)) ;; "i64"
    (when (i32.eq $ty_id $TY_U64)   (call $buf_push_cstr4 $b 3 0x343675)(return)) ;; "u64"
    (when (i32.eq $ty_id $TY_VOID)  (call $buf_push_cstr4 $b 4 0x64696F76)(return)) ;; "void"
    (when (i32.eq $ty_id $TY_NEVER)
      (call $buf_push_cstr4 $b 4 0x6576656E)
      (call $buf_push_u8 $b 114)
      (return))
    (when (i32.eq $ty_id $TY_TUP0)
      (call $buf_push_cstr4 $b 4 0x30707574) ;; "tup0"
      (return))

    (set $t (call $buf_get_i32 $type_table $ty_id))
    (set $kind (i32.load $t))
    (when (i32.eq $kind $TK_LIST)
      (call $buf_push_cstr4 $b 3 0x5F306C) ;; "l0_"
      (call $buf_push_type_slug $b (i32.load offset=4 $t))
      (return))
    (when (i32.eq $kind $TK_BUF)
      (call $buf_push_cstr4 $b 4 0x30667562) ;; "buf0"
      (call $buf_push_u8 $b 95)
      (call $buf_push_type_slug $b (i32.load offset=4 $t))
      (return))
    (when (i32.eq $kind $TK_TUPLE)
      (set $arity (i32.load offset=4 $t))
      (set $sp (i32.load offset=8 $t))
      (call $buf_push_cstr4 $b 3 0x707574) ;; "tup"
      (call $buf_push_u32_dec $b $arity)
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $arity))
          (call $buf_push_u8 $b 95)
          (call $buf_push_type_slug $b (i32.load (i32.add $sp (i32.shl $i 2))))
          (set $i (i32.add $i 1))
          (br $tlp)))
      (return))
    (when (i32.eq $kind $TK_STRUCT)
      (set $sdef (call $buf_get_i32 $all_structs (i32.load offset=4 $t)))
      (set $m (call $buf_get_i32 $modules_buf (i32.load $sdef)))
      (if (i32.gt_u (i32.load offset=4 $m) 0)
        (then
          ;; "st_ns_" + san(ns) + "_" + san(name)
          (call $buf_push_cstr4 $b 4 0x6E5F7473)
          (call $buf_push_cstr4 $b 2 0x5F73)
          (call $buf_push_sanitized $b (i32.load $m) (i32.load offset=4 $m))
          (call $buf_push_u8 $b 95)
          (call $buf_push_sanitized $b (i32.load offset=16 $sdef) (i32.load offset=20 $sdef)))
        (else
          ;; "st_loc_" + san(name)
          (call $buf_push_cstr4 $b 4 0x6C5F7473)
          (call $buf_push_cstr4 $b 3 0x5F636F)
          (call $buf_push_sanitized $b (i32.load offset=16 $sdef) (i32.load offset=20 $sdef))))
      (return))
    (unreachable))

  (func $mod_has_ns (param $dmod i32) (param $ns_p i32) (param $ns_l i32) (result i32)
    (local $m i32)
    (when (i32.eqz $ns_l) (return 0))
    (set $m (call $buf_get_i32 $modules_buf $dmod))
    (call $slice_eq (i32.load $m) (i32.load offset=4 $m) $ns_p $ns_l))

  (func $mods_share_ns (param $m1_idx i32) (param $m2_idx i32) (result i32)
    (local $m1 i32)
    (local $m2 i32)
    (set $m1 (call $buf_get_i32 $modules_buf $m1_idx))
    (when (i32.eqz (i32.load offset=4 $m1)) (return 0))
    (set $m2 (call $buf_get_i32 $modules_buf $m2_idx))
    (call $slice_eq (i32.load $m1) (i32.load offset=4 $m1) (i32.load $m2) (i32.load offset=4 $m2)))

  (func $find_ns_mod (param $ns_p i32) (param $ns_l i32) (result i32)
    (local $cnt i32)
    (local $i   i32)
    (local $m   i32)
    (when (i32.eqz $ns_l) (return -1))
    (set $cnt (call $buf_count_i32 $modules_buf))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $m (call $buf_get_i32 $modules_buf $i))
        (when (call $slice_eq (i32.load $m) (i32.load offset=4 $m) $ns_p $ns_l)
          (return $i))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const -1))

  (func $find_mod_struct_by_name (param $mod_idx i32) (param $req_pub i32) (param $n_p i32) (param $n_l i32) (result i32)
    (local $m    i32)
    (local $sb   i32)
    (local $cnt  i32)
    (local $i    i32)
    (local $sidx i32)
    (local $sdef i32)
    (set $m (call $buf_get_i32 $modules_buf $mod_idx))
    (set $sb (i32.load offset=12 $m))
    (set $cnt (call $buf_count_i32 $sb))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $sidx (call $buf_get_i32 $sb $i))
        (set $sdef (call $buf_get_i32 $all_structs $sidx))
        (when
          (i32.and
            (i32.or (i32.eqz $req_pub) (i32.load offset=4 $sdef))
            (call $slice_eq (i32.load offset=16 $sdef) (i32.load offset=20 $sdef) $n_p $n_l))
          (return $sidx))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const -1))

  (func $find_mod_struct_by_dc (param $mod_idx i32) (param $req_pub i32) (param $dc_p i32) (param $dc_l i32) (result i32)
    (local $m    i32)
    (local $sb   i32)
    (local $cnt  i32)
    (local $i    i32)
    (local $sidx i32)
    (local $sdef i32)
    (when (i32.eqz $dc_l) (return -1))
    (set $m (call $buf_get_i32 $modules_buf $mod_idx))
    (set $sb (i32.load offset=12 $m))
    (set $cnt (call $buf_count_i32 $sb))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $sidx (call $buf_get_i32 $sb $i))
        (set $sdef (call $buf_get_i32 $all_structs $sidx))
        (when
          (i32.and
            (i32.gt_u (i32.load offset=12 $sdef) 0)
            (i32.and
              (i32.or (i32.eqz $req_pub) (i32.load offset=4 $sdef))
              (call $slice_eq (i32.load offset=8 $sdef) (i32.load offset=12 $sdef) $dc_p $dc_l)))
          (return $sidx))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const -1))

  (func $resolve_struct_ident (param $mod_idx i32) (param $n_p i32) (param $n_l i32) (result i32)
    (local $sidx i32)
    (local $m    i32)
    (local $ub   i32)
    (local $cnt  i32)
    (local $i    i32)
    (local $u    i32)
    (local $dmod i32)
    (set $sidx (call $find_mod_struct_by_name $mod_idx 0 $n_p $n_l))
    (when (i32.ge_s $sidx 0) (return $sidx))
    (set $m (call $buf_get_i32 $modules_buf $mod_idx))
    (set $ub (i32.load offset=8 $m))
    (set $cnt (call $buf_count_i32 $ub))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $u (call $buf_get_i32 $ub $i))
        (when (call $slice_eq (i32.load offset=20 $u) (i32.load offset=24 $u) $n_p $n_l)
          (set $dmod 0)
          (block $dm_done
            (loop $dm_lp
              (br_if $dm_done (i32.ge_u $dmod $mod_idx))
              (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                (set $sidx
                  (if (result i32) (i32.load offset=8 $u)
                    (then (call $find_mod_struct_by_dc $dmod 1 (i32.load offset=12 $u) (i32.load offset=16 $u)))
                    (else (call $find_mod_struct_by_name $dmod 1 (i32.load offset=12 $u) (i32.load offset=16 $u)))))
                (when (i32.ge_s $sidx 0) (return $sidx)))
              (set $dmod (i32.add $dmod 1))
              (br $dm_lp))))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const -1))

  (func $resolve_struct_dc (param $mod_idx i32) (param $dc_p i32) (param $dc_l i32) (result i32)
    (local $sidx i32)
    (local $m    i32)
    (local $ub   i32)
    (local $cnt  i32)
    (local $i    i32)
    (local $u    i32)
    (local $dmod i32)
    (set $sidx (call $find_mod_struct_by_dc $mod_idx 0 $dc_p $dc_l))
    (when (i32.ge_s $sidx 0) (return $sidx))
    (set $m (call $buf_get_i32 $modules_buf $mod_idx))
    (set $ub (i32.load offset=8 $m))
    (set $cnt (call $buf_count_i32 $ub))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $u (call $buf_get_i32 $ub $i))
        (when
          (i32.and
            (i32.load offset=8 $u)
            (call $slice_eq (i32.load offset=12 $u) (i32.load offset=16 $u) $dc_p $dc_l))
          (set $dmod 0)
          (block $dm2_done
            (loop $dm2_lp
              (br_if $dm2_done (i32.ge_u $dmod $mod_idx))
              (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                (set $sidx (call $find_mod_struct_by_dc $dmod 1 $dc_p $dc_l))
                (when (i32.ge_s $sidx 0) (return $sidx)))
              (set $dmod (i32.add $dmod 1))
              (br $dm2_lp))))
        (set $i (i32.add $i 1))
        (br $lp)))
    (set $dmod 0)
    (block $gdone
      (loop $glp
        (br_if $gdone (i32.ge_u $dmod $mod_idx))
        (set $sidx (call $find_mod_struct_by_dc $dmod 1 $dc_p $dc_l))
        (when (i32.ge_s $sidx 0) (return $sidx))
        (set $dmod (i32.add $dmod 1))
        (br $glp)))
    (i32.const -1))

  (func $canon_type (param $mod_idx i32) (param $raw_ty i32) (result i32)
    (local $kind  i32)
    (local $arity i32)
    (local $rptr  i32)
    (local $cptr  i32)
    (local $i     i32)
    (local $sidx  i32)
    (set $kind (i32.load $raw_ty))
    (when (i32.le_u $kind 7) (return $kind))
    (when (i32.eq $kind $TK_LIST)
      (return (call $intern_unary_ty $TK_LIST (call $canon_type $mod_idx (i32.load offset=4 $raw_ty)))))
    (when (i32.eq $kind $TK_BUF)
      (return (call $intern_unary_ty $TK_BUF (call $canon_type $mod_idx (i32.load offset=4 $raw_ty)))))
    (when (i32.eq $kind $TK_TUPLE)
      (set $arity (i32.load offset=4 $raw_ty))
      (when (i32.eqz $arity) (return $TY_TUP0))
      (set $rptr (i32.load offset=8 $raw_ty))
      (set $cptr (call $alloc (i32.shl $arity 2)))
      (set $i 0)
      (block $done
        (loop $lp
          (br_if $done (i32.ge_u $i $arity))
          (i32.store
            (i32.add $cptr (i32.shl $i 2))
            (call $canon_type $mod_idx (i32.load (i32.add $rptr (i32.shl $i 2)))))
          (set $i (i32.add $i 1))
          (br $lp)))
      (return (call $intern_tuple_ty $arity $cptr)))
    (when (i32.eq $kind $TK_RAW_SIDENT)
      (set $sidx (call $resolve_struct_ident $mod_idx (i32.load offset=4 $raw_ty) (i32.load offset=8 $raw_ty)))
      (when (i32.lt_s $sidx 0) (unreachable))
      (return (call $intern_struct_ty $sidx)))
    (when (i32.eq $kind $TK_RAW_SDC)
      (set $sidx (call $resolve_struct_dc $mod_idx (i32.load offset=4 $raw_ty) (i32.load offset=8 $raw_ty)))
      (when (i32.lt_s $sidx 0) (unreachable))
      (return (call $intern_struct_ty $sidx)))
    (unreachable))

  ;; Unguarded value-cycle check (`TK_LIST` and `TK_BUF` guard heap indirection;
  ;; direct `TK_STRUCT` and `TK_TUPLE` recursion traps).
  (func $check_struct_value_cycle (param $ty_id i32)
    (local $t     i32)
    (local $kind  i32)
    (local $arity i32)
    (local $sp    i32)
    (local $i     i32)
    (local $sdef  i32)
    (local $state i32)
    (when (i32.le_u $ty_id $TY_TUP0) (return))
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (set $kind (i32.load $t))
    (when (i32.or (i32.eq $kind $TK_LIST) (i32.eq $kind $TK_BUF)) (return))
    (when (i32.eq $kind $TK_TUPLE)
      (set $arity (i32.load offset=4 $t))
      (set $sp (i32.load offset=8 $t))
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $arity))
          (call $check_struct_value_cycle (i32.load (i32.add $sp (i32.shl $i 2))))
          (set $i (i32.add $i 1))
          (br $tlp)))
      (return))
    (when (i32.eq $kind $TK_STRUCT)
      (set $sdef (call $buf_get_i32 $all_structs (i32.load offset=4 $t)))
      (set $state (i32.load offset=36 $sdef))
      (when (i32.eq $state 2) (return))
      (when (i32.eq $state 1) (unreachable))
      (i32.store offset=36 $sdef 1)
      (set $arity (i32.load offset=32 $sdef))
      (set $sp (i32.load offset=28 $sdef))
      (set $i 0)
      (block $sdone
        (loop $slp
          (br_if $sdone (i32.ge_u $i $arity))
          (call $check_struct_value_cycle (i32.load (i32.add $sp (i32.shl $i 2))))
          (set $i (i32.add $i 1))
          (br $slp)))
      (i32.store offset=36 $sdef 2)))

  ;; Checks whether `$ty_id` transitively contains `buf/T`, using `StructDef +40`
  ;; as temporary recursion guard for `l/S`-guarded recursive structs.
  (func $ty_contains_buf (param $ty_id i32) (result i32)
    (local $t     i32)
    (local $kind  i32)
    (local $arity i32)
    (local $sp    i32)
    (local $i     i32)
    (local $sdef  i32)
    (when (i32.le_u $ty_id $TY_TUP0) (return 0))
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (set $kind (i32.load $t))
    (when (i32.eq $kind $TK_BUF) (return 1))
    (when (i32.eq $kind $TK_LIST)
      (return (call $ty_contains_buf (i32.load offset=4 $t))))
    (when (i32.eq $kind $TK_TUPLE)
      (set $arity (i32.load offset=4 $t))
      (set $sp (i32.load offset=8 $t))
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $arity))
          (when (call $ty_contains_buf (i32.load (i32.add $sp (i32.shl $i 2))))
            (return 1))
          (set $i (i32.add $i 1))
          (br $tlp)))
      (return 0))
    (when (i32.eq $kind $TK_STRUCT)
      (set $sdef (call $buf_get_i32 $all_structs (i32.load offset=4 $t)))
      (when (i32.load offset=40 $sdef) (return 0))
      (i32.store offset=40 $sdef 1)
      (set $arity (i32.load offset=32 $sdef))
      (set $sp (i32.load offset=28 $sdef))
      (set $i 0)
      (block $sdone
        (loop $slp
          (br_if $sdone (i32.ge_u $i $arity))
          (when (call $ty_contains_buf (i32.load (i32.add $sp (i32.shl $i 2))))
            (i32.store offset=40 $sdef 0)
            (return 1))
          (set $i (i32.add $i 1))
          (br $slp)))
      (i32.store offset=40 $sdef 0)
      (return 0))
    (unreachable))

  ;; Validates well-formedness of a canonicalized type (`l/T` must not wrap `buf/U`).
  (func $validate_type_wf (param $ty_id i32)
    (local $t     i32)
    (local $kind  i32)
    (local $inner i32)
    (local $arity i32)
    (local $sp    i32)
    (local $i     i32)
    (when (i32.le_u $ty_id $TY_TUP0) (return))
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (set $kind (i32.load $t))
    (when (i32.eq $kind $TK_LIST)
      (set $inner (i32.load offset=4 $t))
      (when (call $ty_contains_buf $inner) (unreachable))
      (call $validate_type_wf $inner)
      (return))
    (when (i32.eq $kind $TK_BUF)
      (call $validate_type_wf (i32.load offset=4 $t))
      (return))
    (when (i32.eq $kind $TK_TUPLE)
      (set $arity (i32.load offset=4 $t))
      (set $sp (i32.load offset=8 $t))
      (set $i 0)
      (block $done
        (loop $lp
          (br_if $done (i32.ge_u $i $arity))
          (call $validate_type_wf (i32.load (i32.add $sp (i32.shl $i 2))))
          (set $i (i32.add $i 1))
          (br $lp)))))

  ;; Computes composite slot layout (`arity * 16` bytes: `+0 off, +4 stride, +8 align_exp, +12 ty_id`)
  ;; and `total_bytes = max(4, align4(cur))` on `TypeEntry` (`+12 = 2, +16 = total_bytes, +20 = lptr`).
  (func $ensure_composite_layout (param $ty_id i32)
    (local $t     i32)
    (local $kind  i32)
    (local $arity i32)
    (local $sp    i32)
    (local $sdef  i32)
    (local $lptr  i32)
    (local $cur   i32)
    (local $i     i32)
    (local $sty   i32)
    (local $str   i32)
    (local $aexp  i32)
    (local $mask  i32)
    (local $off   i32)
    (local $ep    i32)
    (local $tot   i32)

    (when (i32.eq $ty_id $TY_TUP0) (return))
    (when (i32.lt_u $ty_id 8) (unreachable))
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (when (i32.eq (i32.load offset=12 $t) 2) (return))

    (set $kind (i32.load $t))
    (if (i32.eq $kind $TK_TUPLE)
      (then
        (set $arity (i32.load offset=4 $t))
        (set $sp (i32.load offset=8 $t)))
      (else
        (when (i32.ne $kind $TK_STRUCT) (unreachable))
        (set $sdef (call $buf_get_i32 $all_structs (i32.load offset=4 $t)))
        (set $arity (i32.load offset=32 $sdef))
        (set $sp (i32.load offset=28 $sdef))))

    (set $lptr (call $alloc (i32.shl $arity 4)))
    (set $cur 0)
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $arity))
        (set $sty (i32.load (i32.add $sp (i32.shl $i 2))))
        (set $str (call $ty_stride $sty))
        (set $aexp (call $ty_align_exp $sty))
        (set $mask (i32.sub $str 1))
        (set $off (i32.and (i32.add $cur $mask) (i32.xor $mask -1)))
        (set $ep (i32.add $lptr (i32.shl $i 4)))
        (i32.store $ep $off)
        (i32.store offset=4 $ep $str)
        (i32.store offset=8 $ep $aexp)
        (i32.store offset=12 $ep $sty)
        (set $cur (i32.add $off $str))
        (set $i (i32.add $i 1))
        (br $lp)))
    (set $tot (i32.and (i32.add $cur 3) -4))
    (when (i32.lt_u $tot 4) (set $tot 4))
    (i32.store offset=12 $t 2)
    (i32.store offset=16 $t $tot)
    (i32.store offset=20 $t $lptr))

  (func $routine_params_eq (param $r1 i32) (param $r2 i32) (result i32)
    (local $pb1 i32)
    (local $pb2 i32)
    (local $n   i32)
    (local $i   i32)
    (local $p1  i32)
    (local $p2  i32)
    (set $pb1 (i32.load offset=24 $r1))
    (set $pb2 (i32.load offset=24 $r2))
    (set $n (call $buf_count_i32 $pb1))
    (when (i32.ne $n (call $buf_count_i32 $pb2)) (return 0))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $n))
        (set $p1 (call $buf_get_i32 $pb1 $i))
        (set $p2 (call $buf_get_i32 $pb2 $i))
        (when (i32.ne (i32.load offset=12 $p1) (i32.load offset=12 $p2))
          (return 0))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const 1))

  (func $module_exports_use_item (param $dmod i32) (param $u i32) (result i32)
    (local $is_dc i32)
    (local $tp    i32)
    (local $tl    i32)
    (local $m     i32)
    (local $vb    i32)
    (local $rb    i32)
    (local $cnt   i32)
    (local $i     i32)
    (local $tv    i32)
    (local $r     i32)

    (set $is_dc (i32.load offset=8 $u))
    (set $tp (i32.load offset=12 $u))
    (set $tl (i32.load offset=16 $u))
    (if $is_dc
      (then
        (when (i32.ge_s (call $find_mod_struct_by_dc $dmod 1 $tp $tl) 0) (return 1)))
      (else
        (when (i32.ge_s (call $find_mod_struct_by_name $dmod 1 $tp $tl) 0) (return 1))))

    (set $m (call $buf_get_i32 $modules_buf $dmod))
    (set $vb (i32.load offset=16 $m))
    (set $cnt (call $buf_count_i32 $vb))
    (set $i 0)
    (block $vdone
      (loop $vlp
        (br_if $vdone (i32.ge_u $i $cnt))
        (set $tv (call $buf_get_i32 $vb $i))
        (when (i32.load $tv)
          (if $is_dc
            (then
              (when (call $slice_eq (i32.load offset=4 $tv) (i32.load offset=8 $tv) $tp $tl)
                (return 1)))
            (else
              (when (call $slice_eq (i32.load offset=16 $tv) (i32.load offset=20 $tv) $tp $tl)
                (return 1)))))
        (set $i (i32.add $i 1))
        (br $vlp)))

    (set $rb (i32.load offset=20 $m))
    (set $cnt (call $buf_count_i32 $rb))
    (set $i 0)
    (block $rdone
      (loop $rlp
        (br_if $rdone (i32.ge_u $i $cnt))
        (set $r (call $buf_get_i32 $rb $i))
        (when (i32.load $r)
          (if $is_dc
            (then
              (when (call $slice_eq (i32.load offset=4 $r) (i32.load offset=8 $r) $tp $tl)
                (return 1)))
            (else
              (when (call $slice_eq (i32.load offset=16 $r) (i32.load offset=20 $r) $tp $tl)
                (return 1)))))
        (set $i (i32.add $i 1))
        (br $rlp)))
    (i32.const 0))

  (func $canonicalize_all_declarations
    (local $mcnt     i32)
    (local $mi       i32)
    (local $mj       i32)
    (local $m        i32)
    (local $m2       i32)
    (local $scnt     i32)
    (local $si       i32)
    (local $sj       i32)
    (local $sdef     i32)
    (local $sdef2    i32)
    (local $rfb      i32)
    (local $arity    i32)
    (local $cfp      i32)
    (local $fi       i32)
    (local $fty      i32)
    (local $ub       i32)
    (local $ucnt     i32)
    (local $ui       i32)
    (local $u        i32)
    (local $dmod     i32)
    (local $vb       i32)
    (local $vcnt     i32)
    (local $vi       i32)
    (local $vj       i32)
    (local $tv       i32)
    (local $tv2      i32)
    (local $cty      i32)
    (local $vt       i32)
    (local $g_seq    i32)
    (local $gn_p     i32)
    (local $rb       i32)
    (local $rcnt     i32)
    (local $ri       i32)
    (local $rj       i32)
    (local $r        i32)
    (local $r2       i32)
    (local $pb       i32)
    (local $pcnt     i32)
    (local $pi       i32)
    (local $param    i32)
    (local $pty      i32)
    (local $name_cnt i32)
    (local $fn_buf   i32)
    (local $found_exp i32)
    (local $vb2      i32)
    (local $vcnt2    i32)
    (local $rb2      i32)
    (local $rcnt2    i32)

    (set $mcnt (call $buf_count_i32 $modules_buf))
    (when (i32.eqz $mcnt) (unreachable))

    ;; 1. Verify valid module-level `use` targets across all prior strata `0..$mi`
    (set $mi 0)
    (block $m1_done
      (loop $m1_lp
        (br_if $m1_done (i32.ge_u $mi $mcnt))
        (set $m (call $buf_get_i32 $modules_buf $mi))
        (set $ub (i32.load offset=8 $m))
        (set $ucnt (call $buf_count_i32 $ub))
        (set $ui 0)
        (block $u_done
          (loop $u_lp
            (br_if $u_done (i32.ge_u $ui $ucnt))
            (set $u (call $buf_get_i32 $ub $ui))
            (set $found_exp 0)
            (set $dmod 0)
            (block $dm_chk_done
              (loop $dm_chk_lp
                (br_if $dm_chk_done (i32.ge_u $dmod $mi))
                (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                  (when (call $module_exports_use_item $dmod $u)
                    (set $found_exp 1)
                    (br $dm_chk_done)))
                (set $dmod (i32.add $dmod 1))
                (br $dm_chk_lp)))
            (when (i32.eqz $found_exp) (unreachable))
            (set $ui (i32.add $ui 1))
            (br $u_lp)))
        (set $mi (i32.add $mi 1))
        (br $m1_lp)))

    ;; 2. Canonicalize all `StructDef` fields & check intra-module and cross-stratum `pub` duplicates
    (set $scnt (call $buf_count_i32 $all_structs))
    (set $si 0)
    (block $s1_done
      (loop $s1_lp
        (br_if $s1_done (i32.ge_u $si $scnt))
        (set $sdef (call $buf_get_i32 $all_structs $si))
        (set $sj (i32.add $si 1))
        (block $sdup_done
          (loop $sdup_lp
            (br_if $sdup_done (i32.ge_u $sj $scnt))
            (set $sdef2 (call $buf_get_i32 $all_structs $sj))
            (when
              (i32.or
                (i32.eq (i32.load $sdef) (i32.load $sdef2))
                (i32.and
                  (i32.and (i32.load offset=4 $sdef) (i32.load offset=4 $sdef2))
                  (call $mods_share_ns (i32.load $sdef) (i32.load $sdef2))))
              (when
                (call $slice_eq
                  (i32.load offset=16 $sdef) (i32.load offset=20 $sdef)
                  (i32.load offset=16 $sdef2) (i32.load offset=20 $sdef2))
                (unreachable)))
            (set $sj (i32.add $sj 1))
            (br $sdup_lp)))
        (set $rfb (i32.load offset=24 $sdef))
        (set $arity (call $buf_count_i32 $rfb))
        (set $cfp (call $alloc (i32.shl $arity 2)))
        (set $fi 0)
        (block $sf_done
          (loop $sf_lp
            (br_if $sf_done (i32.ge_u $fi $arity))
            (set $fty (call $canon_type (i32.load $sdef) (call $buf_get_i32 $rfb $fi)))
            (when (i32.or (i32.eq $fty $TY_VOID) (i32.eq $fty $TY_NEVER)) (unreachable))
            (i32.store (i32.add $cfp (i32.shl $fi 2)) $fty)
            (set $fi (i32.add $fi 1))
            (br $sf_lp)))
        (i32.store offset=28 $sdef $cfp)
        (i32.store offset=32 $sdef $arity)
        (set $si (i32.add $si 1))
        (br $s1_lp)))

    ;; 3. Check unguarded value cycles & `buf/T` well-formedness on structs
    (set $si 0)
    (block $s2_done
      (loop $s2_lp
        (br_if $s2_done (i32.ge_u $si $scnt))
        (call $check_struct_value_cycle (call $intern_struct_ty $si))
        (set $sdef (call $buf_get_i32 $all_structs $si))
        (set $arity (i32.load offset=32 $sdef))
        (set $cfp (i32.load offset=28 $sdef))
        (set $fi 0)
        (block $swf_done
          (loop $swf_lp
            (br_if $swf_done (i32.ge_u $fi $arity))
            (set $fty (i32.load (i32.add $cfp (i32.shl $fi 2))))
            (call $validate_type_wf $fty)
            (when (i32.and (i32.load offset=4 $sdef) (call $ty_contains_buf $fty))
              (unreachable))
            (set $fi (i32.add $fi 1))
            (br $swf_lp)))
        (set $si (i32.add $si 1))
        (br $s2_lp)))

    ;; 4. Canonicalize `TopVal`s & `RoutineAst`s and reserve WASM globals/functions
    (set $g_seq 0)
    (set $mi 0)
    (block $m2_done
      (loop $m2_lp
        (br_if $m2_done (i32.ge_u $mi $mcnt))
        (set $m (call $buf_get_i32 $modules_buf $mi))

        ;; Top-level constants
        (set $vb (i32.load offset=16 $m))
        (set $vcnt (call $buf_count_i32 $vb))
        (set $vi 0)
        (block $v_done
          (loop $v_lp
            (br_if $v_done (i32.ge_u $vi $vcnt))
            (set $tv (call $buf_get_i32 $vb $vi))
            (set $cty (call $canon_type $mi (i32.load offset=12 $tv)))
            (call $validate_type_wf $cty)
            (when (call $ty_contains_buf $cty) (unreachable))
            (set $vt (call $ty_wasm_vt $cty))
            (when (i32.eqz $vt) (unreachable))
            (i32.store offset=28 $tv $cty)
            (set $vj 0)
            (block $vdup_done
              (loop $vdup_lp
                (br_if $vdup_done (i32.ge_u $vj $vi))
                (set $tv2 (call $buf_get_i32 $vb $vj))
                (when
                  (i32.and
                    (i32.eq (i32.load offset=28 $tv2) $cty)
                    (call $slice_eq
                      (i32.load offset=16 $tv) (i32.load offset=20 $tv)
                      (i32.load offset=16 $tv2) (i32.load offset=20 $tv2)))
                  (unreachable))
                (set $vj (i32.add $vj 1))
                (br $vdup_lp)))
            (set $gn_p (call $make_tag_name 99 $g_seq)) ;; "__c_<g_seq>"
            (set $g_seq (i32.add $g_seq 1))
            (i32.store offset=32 $tv (call $ensure_global_vt $gn_p $ret_len $vt))
            (set $vi (i32.add $vi 1))
            (br $v_lp)))

        ;; Top-level routines (Pass A: canonicalize param/return types)
        (set $rb (i32.load offset=20 $m))
        (set $rcnt (call $buf_count_i32 $rb))
        (set $ri 0)
        (block $ra_done
          (loop $ra_lp
            (br_if $ra_done (i32.ge_u $ri $rcnt))
            (set $r (call $buf_get_i32 $rb $ri))
            (set $cty (call $canon_type $mi (i32.load offset=12 $r)))
            (call $validate_type_wf $cty)
            (when (i32.and (i32.load $r) (call $ty_contains_buf $cty)) (unreachable))
            (i32.store offset=32 $r $cty)
            (set $pb (i32.load offset=24 $r))
            (set $pcnt (call $buf_count_i32 $pb))
            (set $pi 0)
            (block $p_done
              (loop $p_lp
                (br_if $p_done (i32.ge_u $pi $pcnt))
                (set $param (call $buf_get_i32 $pb $pi))
                (set $pty (call $canon_type $mi (i32.load $param)))
                (call $validate_type_wf $pty)
                (when (i32.eqz (call $ty_wasm_vt $pty)) (unreachable))
                (when (i32.and (i32.load $r) (call $ty_contains_buf $pty)) (unreachable))
                (i32.store offset=12 $param $pty)
                (set $pi (i32.add $pi 1))
                (br $p_lp)))
            (set $ri (i32.add $ri 1))
            (br $ra_lp)))

        ;; Top-level routines (Pass B: check overload uniqueness & reserve `sl_fn_...`)
        (set $ri 0)
        (block $rb_done
          (loop $rb_lp
            (br_if $rb_done (i32.ge_u $ri $rcnt))
            (set $r (call $buf_get_i32 $rb $ri))
            (set $name_cnt 0)
            (set $rj 0)
            (block $rc_done
              (loop $rc_lp
                (br_if $rc_done (i32.ge_u $rj $rcnt))
                (set $r2 (call $buf_get_i32 $rb $rj))
                (when
                  (call $slice_eq
                    (i32.load offset=16 $r) (i32.load offset=20 $r)
                    (i32.load offset=16 $r2) (i32.load offset=20 $r2))
                  (set $name_cnt (i32.add $name_cnt 1))
                  (when (i32.and (i32.ne $ri $rj) (call $routine_params_eq $r $r2))
                    (unreachable)))
                (set $rj (i32.add $rj 1))
                (br $rc_lp)))

            ;; Format `sl_fn_m<mi>[_<ns>]_<san>[__<params_slug>]`
            (set $fn_buf (call $buf_new 48))
            (call $buf_push_cstr4 $fn_buf 4 0x665F6C73) ;; "sl_f"
            (call $buf_push_cstr4 $fn_buf 3 0x6D5F6E)   ;; "n_m"
            (call $buf_push_u32_dec $fn_buf $mi)
            (when (i32.gt_u (i32.load offset=4 $m) 0)
              (call $buf_push_u8 $fn_buf 95)
              (call $buf_push_sanitized $fn_buf (i32.load $m) (i32.load offset=4 $m)))
            (call $buf_push_u8 $fn_buf 95)
            (call $buf_push_sanitized $fn_buf (i32.load offset=16 $r) (i32.load offset=20 $r))
            (when (i32.gt_u $name_cnt 1)
              (call $buf_push_cstr4 $fn_buf 2 0x5F5F) ;; "__"
              (set $pb (i32.load offset=24 $r))
              (set $pcnt (call $buf_count_i32 $pb))
              (if (i32.eqz $pcnt)
                (then (call $buf_push_u8 $fn_buf 48))
                (else
                  (set $pi 0)
                  (block $ps_done
                    (loop $ps_lp
                      (br_if $ps_done (i32.ge_u $pi $pcnt))
                      (when (i32.gt_u $pi 0)
                        (call $buf_push_cstr4 $fn_buf 2 0x5F5F))
                      (set $param (call $buf_get_i32 $pb $pi))
                      (call $buf_push_type_slug $fn_buf (i32.load offset=12 $param))
                      (set $pi (i32.add $pi 1))
                      (br $ps_lp))))))
            (i32.store offset=36 $r
              (call $reserve_routine (i32.load $fn_buf) (i32.load offset=4 $fn_buf)))
            (set $ri (i32.add $ri 1))
            (br $rb_lp)))

        (set $mi (i32.add $mi 1))
        (br $m2_lp)))

    ;; 4b. Verify cross-stratum `pub` uniqueness for modules sharing a namespace
    (set $mi 0)
    (block $cs1_done
      (loop $cs1_lp
        (br_if $cs1_done (i32.ge_u $mi $mcnt))
        (set $m (call $buf_get_i32 $modules_buf $mi))
        (when (i32.gt_u (i32.load offset=4 $m) 0)
          (set $mj (i32.add $mi 1))
          (block $cs2_done
            (loop $cs2_lp
              (br_if $cs2_done (i32.ge_u $mj $mcnt))
              (when (call $mods_share_ns $mi $mj)
                (set $m2 (call $buf_get_i32 $modules_buf $mj))
                (set $vb (i32.load offset=16 $m))
                (set $vcnt (call $buf_count_i32 $vb))
                (set $vb2 (i32.load offset=16 $m2))
                (set $vcnt2 (call $buf_count_i32 $vb2))
                (set $vi 0)
                (block $cv1_done
                  (loop $cv1_lp
                    (br_if $cv1_done (i32.ge_u $vi $vcnt))
                    (set $tv (call $buf_get_i32 $vb $vi))
                    (when (i32.load $tv)
                      (set $vj 0)
                      (block $cv2_done
                        (loop $cv2_lp
                          (br_if $cv2_done (i32.ge_u $vj $vcnt2))
                          (set $tv2 (call $buf_get_i32 $vb2 $vj))
                          (when
                            (i32.and
                              (i32.load $tv2)
                              (call $slice_eq
                                (i32.load offset=16 $tv) (i32.load offset=20 $tv)
                                (i32.load offset=16 $tv2) (i32.load offset=20 $tv2)))
                            (unreachable))
                          (set $vj (i32.add $vj 1))
                          (br $cv2_lp))))
                    (set $vi (i32.add $vi 1))
                    (br $cv1_lp)))
                (set $rb (i32.load offset=20 $m))
                (set $rcnt (call $buf_count_i32 $rb))
                (set $rb2 (i32.load offset=20 $m2))
                (set $rcnt2 (call $buf_count_i32 $rb2))
                (set $ri 0)
                (block $cr1_done
                  (loop $cr1_lp
                    (br_if $cr1_done (i32.ge_u $ri $rcnt))
                    (set $r (call $buf_get_i32 $rb $ri))
                    (when (i32.load $r)
                      (set $rj 0)
                      (block $cr2_done
                        (loop $cr2_lp
                          (br_if $cr2_done (i32.ge_u $rj $rcnt2))
                          (set $r2 (call $buf_get_i32 $rb2 $rj))
                          (when
                            (i32.and
                              (i32.load $r2)
                              (call $slice_eq
                                (i32.load offset=16 $r) (i32.load offset=20 $r)
                                (i32.load offset=16 $r2) (i32.load offset=20 $r2)))
                            (when
                              (i32.or
                                (i32.ne
                                  (call $buf_count_i32 (i32.load offset=24 $r))
                                  (call $buf_count_i32 (i32.load offset=24 $r2)))
                                (call $routine_params_eq $r $r2))
                              (unreachable)))
                          (set $rj (i32.add $rj 1))
                          (br $cr2_lp))))
                    (set $ri (i32.add $ri 1))
                    (br $cr1_lp))))
              (set $mj (i32.add $mj 1))
              (br $cs2_lp))))
        (set $mi (i32.add $mi 1))
        (br $cs1_lp))))

  ;; Resolves a top-level constant in `$dmod` matching `($req_pub, $opt_ty_plus1, $is_dc, $p, $l)`.
  ;; Returns `TopVal*` (`0` if none, traps if ambiguous >1).
  (func $find_mod_top_val
    (param $dmod i32) (param $req_pub i32) (param $opt_ty_plus1 i32) (param $is_dc i32) (param $p i32) (param $l i32) (result i32)
    (local $m     i32)
    (local $vb    i32)
    (local $cnt   i32)
    (local $i     i32)
    (local $tv    i32)
    (local $found i32)
    (local $ok    i32)
    (set $m (call $buf_get_i32 $modules_buf $dmod))
    (set $vb (i32.load offset=16 $m))
    (set $cnt (call $buf_count_i32 $vb))
    (set $found 0)
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $tv (call $buf_get_i32 $vb $i))
        (set $ok (i32.or (i32.eqz $req_pub) (i32.load $tv)))
        (when (i32.and $ok (i32.gt_u $opt_ty_plus1 0))
          (when (i32.ne (i32.load offset=28 $tv) (i32.sub $opt_ty_plus1 1))
            (set $ok 0)))
        (when $ok
          (if $is_dc
            (then
              (set $ok
                (i32.and
                  (i32.gt_u (i32.load offset=8 $tv) 0)
                  (call $slice_eq (i32.load offset=4 $tv) (i32.load offset=8 $tv) $p $l))))
            (else
              (set $ok (call $slice_eq (i32.load offset=16 $tv) (i32.load offset=20 $tv) $p $l)))))
        (when $ok
          (when $found (unreachable))
          (set $found $tv))
        (set $i (i32.add $i 1))
        (br $lp)))
    (get $found))

  (func $resolve_top_val (param $mod_idx i32) (param $opt_ty_plus1 i32) (param $is_dc i32) (param $p i32) (param $l i32) (result i32)
    (local $tv   i32)
    (local $m    i32)
    (local $ub   i32)
    (local $cnt  i32)
    (local $i    i32)
    (local $u    i32)
    (local $dmod i32)
    (local $cand i32)
    (set $tv (call $find_mod_top_val $mod_idx 0 $opt_ty_plus1 $is_dc $p $l))
    (when $tv (return $tv))
    (set $m (call $buf_get_i32 $modules_buf $mod_idx))
    (set $ub (i32.load offset=8 $m))
    (set $cnt (call $buf_count_i32 $ub))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $u (call $buf_get_i32 $ub $i))
        (if $is_dc
          (then
            (when
              (i32.and
                (i32.load offset=8 $u)
                (call $slice_eq (i32.load offset=12 $u) (i32.load offset=16 $u) $p $l))
              (set $dmod 0)
              (block $vd1_done
                (loop $vd1_lp
                  (br_if $vd1_done (i32.ge_u $dmod $mod_idx))
                  (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                    (set $cand (call $find_mod_top_val $dmod 1 $opt_ty_plus1 1 $p $l))
                    (when $cand
                      (when (i32.and $tv (i32.ne $tv $cand)) (unreachable))
                      (set $tv $cand)))
                  (set $dmod (i32.add $dmod 1))
                  (br $vd1_lp)))))
          (else
            (when (call $slice_eq (i32.load offset=20 $u) (i32.load offset=24 $u) $p $l)
              (set $dmod 0)
              (block $vd2_done
                (loop $vd2_lp
                  (br_if $vd2_done (i32.ge_u $dmod $mod_idx))
                  (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                    (set $cand
                      (call $find_mod_top_val
                        $dmod 1 $opt_ty_plus1
                        (i32.load offset=8 $u) (i32.load offset=12 $u) (i32.load offset=16 $u)))
                    (when $cand
                      (when (i32.and $tv (i32.ne $tv $cand)) (unreachable))
                      (set $tv $cand)))
                  (set $dmod (i32.add $dmod 1))
                  (br $vd2_lp))))))
        (set $i (i32.add $i 1))
        (br $lp)))
    (when (i32.and $is_dc (i32.eqz $tv))
      (set $dmod 0)
      (block $gdone
        (loop $glp
          (br_if $gdone (i32.ge_u $dmod $mod_idx))
          (set $cand (call $find_mod_top_val $dmod 1 $opt_ty_plus1 1 $p $l))
          (when $cand
            (when (i32.and $tv (i32.ne $tv $cand)) (unreachable))
            (set $tv $cand))
          (set $dmod (i32.add $dmod 1))
          (br $glp))))
    (when (i32.eqz $tv) (unreachable))
    (get $tv))

  (func $routine_matches_args (param $r i32) (param $arity i32) (param $arg_tys_ptr i32) (result i32)
    (local $pb  i32)
    (local $i   i32)
    (local $aty i32)
    (local $pty i32)
    (set $pb (i32.load offset=24 $r))
    (when (i32.ne (call $buf_count_i32 $pb) $arity) (return 0))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $arity))
        (set $aty (i32.load (i32.add $arg_tys_ptr (i32.shl $i 2))))
        (set $pty (i32.load offset=12 (call $buf_get_i32 $pb $i)))
        (when (i32.and (i32.ne $aty $TY_NEVER) (i32.ne $aty $pty))
          (return 0))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const 1))

  (func $find_mod_top_rout
    (param $dmod i32) (param $req_pub i32) (param $is_dc i32) (param $p i32) (param $l i32) (param $arity i32) (param $arg_tys_ptr i32) (result i32)
    (local $m     i32)
    (local $rb    i32)
    (local $cnt   i32)
    (local $i     i32)
    (local $r     i32)
    (local $found i32)
    (local $ok    i32)
    (set $m (call $buf_get_i32 $modules_buf $dmod))
    (set $rb (i32.load offset=20 $m))
    (set $cnt (call $buf_count_i32 $rb))
    (set $found 0)
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $r (call $buf_get_i32 $rb $i))
        (set $ok (i32.or (i32.eqz $req_pub) (i32.load $r)))
        (when $ok
          (if $is_dc
            (then
              (set $ok
                (i32.and
                  (i32.gt_u (i32.load offset=8 $r) 0)
                  (call $slice_eq (i32.load offset=4 $r) (i32.load offset=8 $r) $p $l))))
            (else
              (set $ok (call $slice_eq (i32.load offset=16 $r) (i32.load offset=20 $r) $p $l)))))
        (when (i32.and $ok (call $routine_matches_args $r $arity $arg_tys_ptr))
          (when $found (unreachable))
          (set $found $r))
        (set $i (i32.add $i 1))
        (br $lp)))
    (get $found))

  (func $resolve_top_rout
    (param $mod_idx i32) (param $is_dc i32) (param $p i32) (param $l i32) (param $arity i32) (param $arg_tys_ptr i32) (result i32)
    (local $r    i32)
    (local $m    i32)
    (local $ub   i32)
    (local $cnt  i32)
    (local $i    i32)
    (local $u    i32)
    (local $dmod i32)
    (local $cand i32)
    (set $r (call $find_mod_top_rout $mod_idx 0 $is_dc $p $l $arity $arg_tys_ptr))
    (when $r (return $r))
    (set $m (call $buf_get_i32 $modules_buf $mod_idx))
    (set $ub (i32.load offset=8 $m))
    (set $cnt (call $buf_count_i32 $ub))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $u (call $buf_get_i32 $ub $i))
        (if $is_dc
          (then
            (when
              (i32.and
                (i32.load offset=8 $u)
                (call $slice_eq (i32.load offset=12 $u) (i32.load offset=16 $u) $p $l))
              (set $dmod 0)
              (block $rd1_done
                (loop $rd1_lp
                  (br_if $rd1_done (i32.ge_u $dmod $mod_idx))
                  (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                    (set $cand (call $find_mod_top_rout $dmod 1 1 $p $l $arity $arg_tys_ptr))
                    (when $cand
                      (when (i32.and $r (i32.ne $r $cand)) (unreachable))
                      (set $r $cand)))
                  (set $dmod (i32.add $dmod 1))
                  (br $rd1_lp)))))
          (else
            (when (call $slice_eq (i32.load offset=20 $u) (i32.load offset=24 $u) $p $l)
              (set $dmod 0)
              (block $rd2_done
                (loop $rd2_lp
                  (br_if $rd2_done (i32.ge_u $dmod $mod_idx))
                  (when (call $mod_has_ns $dmod (i32.load $u) (i32.load offset=4 $u))
                    (set $cand
                      (call $find_mod_top_rout
                        $dmod 1
                        (i32.load offset=8 $u) (i32.load offset=12 $u) (i32.load offset=16 $u)
                        $arity $arg_tys_ptr))
                    (when $cand
                      (when (i32.and $r (i32.ne $r $cand)) (unreachable))
                      (set $r $cand)))
                  (set $dmod (i32.add $dmod 1))
                  (br $rd2_lp))))))
        (set $i (i32.add $i 1))
        (br $lp)))
    (when (i32.and $is_dc (i32.eqz $r))
      (set $dmod 0)
      (block $gdone
        (loop $glp
          (br_if $gdone (i32.ge_u $dmod $mod_idx))
          (set $cand (call $find_mod_top_rout $dmod 1 1 $p $l $arity $arg_tys_ptr))
          (when $cand
            (when (i32.and $r (i32.ne $r $cand)) (unreachable))
            (set $r $cand))
          (set $dmod (i32.add $dmod 1))
          (br $glp))))
    (when (i32.eqz $r) (unreachable))
    (get $r))

  ;; ---------------------------------------------------------------------------
  ;; 7. `.WASM` Routine Builder (`FuncBuf`) & Module Serializer
  ;; ---------------------------------------------------------------------------

  ;; FuncBuf (24 bytes):
  ;; +0 pnames_buf (NameSlice* `+0 ptr, +4 len`), +4 pvts_buf (`i32` vt_cp),
  ;; +8 ret_vt_cp, +12 lvts_buf (`i32` vt_cp), +16 body_buf (DcTF bytes)
  (func $fb_new (param $ret_vt_cp i32) (result i32)
    (local $fb i32)
    (set $fb (call $alloc 24))
    (i32.store $fb (call $buf_new 16))
    (i32.store offset=4 $fb (call $buf_new 16))
    (i32.store offset=8 $fb $ret_vt_cp)
    (i32.store offset=12 $fb (call $buf_new 16))
    (i32.store offset=16 $fb (call $buf_new 128))
    (get $fb))

  (func $fb_add_param (param $fb i32) (param $n_p i32) (param $n_l i32) (param $vt_cp i32) (result i32)
    (local $idx i32)
    (local $ns  i32)
    (set $idx (call $buf_count_i32 (i32.load offset=4 $fb)))
    (set $ns (call $alloc 8))
    (i32.store $ns $n_p)
    (i32.store offset=4 $ns $n_l)
    (call $buf_push_i32 (i32.load $fb) $ns)
    (call $buf_push_i32 (i32.load offset=4 $fb) $vt_cp)
    (get $idx))

  (func $fb_alloc_local (param $fb i32) (param $vt_cp i32) (result i32)
    (local $idx i32)
    (set $idx
      (i32.add
        (call $buf_count_i32 (i32.load offset=4 $fb))
        (call $buf_count_i32 (i32.load offset=12 $fb))))
    (call $buf_push_i32 (i32.load offset=12 $fb) $vt_cp)
    (get $idx))

  (func $fb_op (param $fb i32) (param $sub i32)
    (call $emit_cp (i32.load offset=16 $fb) (i32.add $DC_WASM_START $sub)))

  (func $fb_op_u32 (param $fb i32) (param $sub i32) (param $v i32)
    (local $b i32)
    (set $b (i32.load offset=16 $fb))
    (call $emit_cp $b (i32.add $DC_WASM_START $sub))
    (call $emit_u64_dc_num $b (i64.extend_i32_u $v)))

  (func $fb_block (param $fb i32) (param $sub i32) (param $bt_vt_cp i32)
    (local $b i32)
    (set $b (i32.load offset=16 $fb))
    (call $emit_cp $b (i32.add $DC_WASM_START $sub))
    (call $emit_cp $b
      (if (result i32) $bt_vt_cp (then (get $bt_vt_cp)) (else (i32.const $DC_OPTIONAL_ABSENT)))))

  (func $fb_mem (param $fb i32) (param $sub i32) (param $align_exp i32) (param $offset i32)
    (local $b i32)
    (set $b (i32.load offset=16 $fb))
    (call $emit_cp $b (i32.add $DC_WASM_START $sub))
    (call $emit_u64_dc_num $b 0)
    (call $emit_u64_dc_num $b (i64.extend_i32_u $align_exp))
    (call $emit_u64_dc_num $b (i64.extend_i32_u $offset)))

  (func $fb_i32_const (param $fb i32) (param $v i32)
    (local $b i32)
    (set $b (i32.load offset=16 $fb))
    (call $emit_cp $b (i32.add $DC_WASM_START 0x41))
    (call $emit_i64_dc_num $b (i64.extend_i32_s $v)))

  (func $fb_i64_const (param $fb i32) (param $v i64)
    (local $b i32)
    (set $b (i32.load offset=16 $fb))
    (call $emit_cp $b (i32.add $DC_WASM_START 0x42))
    (call $emit_i64_dc_num $b $v))

  (func $fb_mem_copy (param $fb i32)
    (local $b i32)
    (set $b (i32.load offset=16 $fb))
    (call $emit_cp $b (i32.add $DC_WASM_FC_START 10))
    (call $emit_u64_dc_num $b 0)
    (call $emit_u64_dc_num $b 0))

  (func $emit_wasm_ident (param $out i32) (param $kt_p i32) (param $kt_l i32) (param $n_p i32) (param $n_l i32)
    (call $emit_cp $out $DC_BEGIN_IDENTIFIER)
    (when (i32.gt_u $kt_l 0)
      (call $buf_push_bytes $out $kt_p $kt_l))
    ;; Identifier characters are ASCII / canonical DcTF Unicode bytes (no control Dcs)
    (call $buf_push_bytes $out $n_p $n_l)
    (call $emit_cp $out $DC_END_IDENTIFIER))

  (func $fb_finish (param $fb i32) (result i32)
    (local $out    i32)
    (local $pnames i32)
    (local $pvts   i32)
    (local $ret_vt i32)
    (local $lvts   i32)
    (local $body   i32)
    (local $pcnt   i32)
    (local $lcnt   i32)
    (local $i      i32)
    (local $ns     i32)
    (local $kt_b   i32)

    (set $pnames (i32.load $fb))
    (set $pvts (i32.load offset=4 $fb))
    (set $ret_vt (i32.load offset=8 $fb))
    (set $lvts (i32.load offset=12 $fb))
    (set $body (i32.load offset=16 $fb))
    (set $pcnt (call $buf_count_i32 $pvts))
    (set $lcnt (call $buf_count_i32 $lvts))

    (set $out (call $buf_new (i32.add 64 (i32.load offset=4 $body))))
    (call $emit_cp $out $DC_BEGIN_LITERAL)
    (call $emit_cp $out $DC_BEGIN_TYPE_DEF)
    (call $emit_cp $out $DC_TYPE_ROUTINE)

    (if $ret_vt
      (then (call $emit_cp $out $ret_vt))
      (else
        (when (i32.gt_u $pcnt 0)
          (call $emit_cp $out $DC_OPTIONAL_ABSENT))))

    (when (i32.gt_u $pcnt 0)
      (call $emit_cp $out $DC_BEGIN_LIST)
      (call $emit_cp $out $DC_TYPE_ANY)
      (set $i 0)
      (block $pdone
        (loop $plp
          (br_if $pdone (i32.ge_u $i $pcnt))
          (when (i32.gt_u $i 0)
            (call $emit_cp $out $DC_LIST_SEP))
          (set $ns (call $buf_get_i32 $pnames $i))
          (set $kt_b (call $buf_new 8))
          (call $emit_cp $kt_b (call $buf_get_i32 $pvts $i))
          (call $emit_wasm_ident
            $out
            (i32.load $kt_b) (i32.load offset=4 $kt_b)
            (i32.load $ns) (i32.load offset=4 $ns))
          (set $i (i32.add $i 1))
          (br $plp)))
      (call $emit_cp $out $DC_END_LIST))

    (call $emit_cp $out $DC_END_TYPE_DEF)

    (when (i32.gt_u $lcnt 0)
      (call $emit_cp $out $DC_BEGIN_LIST)
      (call $emit_cp $out $DC_ENTITY_TYPE)
      (set $i 0)
      (block $ldone
        (loop $llp
          (br_if $ldone (i32.ge_u $i $lcnt))
          (when (i32.gt_u $i 0)
            (call $emit_cp $out $DC_LIST_SEP))
          (call $emit_cp $out (call $buf_get_i32 $lvts $i))
          (set $i (i32.add $i 1))
          (br $llp)))
      (call $emit_cp $out $DC_END_LIST))

    (call $buf_push_bytes $out (i32.load $body) (i32.load offset=4 $body))
    (call $emit_cp $out $DC_END_LITERAL)
    (global.set $ret_len (i32.load offset=4 $out))
    (i32.load $out))

  (func $serialize_wasm_module (result i32)
    (local $out  i32)
    (local $gcnt i32)
    (local $rcnt i32)
    (local $i    i32)
    (local $g    i32)
    (local $r    i32)

    (set $out (call $buf_new 512))
    (call $emit_cp $out $DC_BEGIN_WASM_PROGRAM)
    (call $emit_cp $out $DC_BEGIN_KV_MAP)
    (call $emit_cp $out $DC_TYPE_ANY)
    (call $emit_cp $out $DC_TYPE_ANY)

    (set $gcnt (call $buf_count_i32 $out_globals))
    (set $i 0)
    (block $gdone
      (loop $glp
        (br_if $gdone (i32.ge_u $i $gcnt))
        (set $g (call $buf_get_i32 $out_globals $i))
        (call $emit_cp $out $DC_BEGIN_KV_PAIR)
        (call $emit_wasm_ident
          $out
          (i32.load offset=8 $g) (i32.load offset=12 $g)
          (i32.load $g) (i32.load offset=4 $g))
        (call $emit_cp $out $DC_LIST_SEP)
        (call $buf_push_bytes $out (i32.load offset=16 $g) (i32.load offset=20 $g))
        (call $emit_cp $out $DC_END_KV_PAIR)
        (set $i (i32.add $i 1))
        (br $glp)))

    (set $rcnt (call $buf_count_i32 $out_routines))
    (set $i 0)
    (block $rdone
      (loop $rlp
        (br_if $rdone (i32.ge_u $i $rcnt))
        (set $r (call $buf_get_i32 $out_routines $i))
        (when (i32.eqz (i32.load offset=12 $r)) (unreachable))
        (call $emit_cp $out $DC_BEGIN_KV_PAIR)
        (call $emit_wasm_ident $out 0 0 (i32.load $r) (i32.load offset=4 $r))
        (call $emit_cp $out $DC_LIST_SEP)
        (call $buf_push_bytes $out (i32.load offset=8 $r) (i32.load offset=12 $r))
        (call $emit_cp $out $DC_END_KV_PAIR)
        (set $i (i32.add $i 1))
        (br $rlp)))

    (call $emit_cp $out $DC_END_KV_MAP)
    (call $emit_cp $out $DC_END_WASM_PROGRAM)
    (global.set $ret_len (i32.load offset=4 $out))
    (i32.load $out))

  ;; ---------------------------------------------------------------------------
  ;; 8. Runtime Helpers (`__sl_shadow_push`, Structural Equality, Stage 0 Links)
  ;; ---------------------------------------------------------------------------

  (func $reserve_runtime_helpers
    (local $p i32)
    (when $uses_shadow
      (set $p (call $make_ascii 13 0x776F646168735F5F 0x657361625F)) ;; "__shadow_base"
      (global.set $sh_base_glob (call $ensure_global_vt $p 13 $FMT_I32))
      (set $p (call $make_ascii 14 0x776F646168735F5F 0x74696D696C5F)) ;; "__shadow_limit"
      (global.set $sh_lim_glob (call $ensure_global_vt $p 14 $FMT_I32))
      (set $p (call $make_ascii 11 0x776F646168735F5F 0x70735F)) ;; "__shadow_sp"
      (global.set $sh_sp_glob (call $ensure_global_vt $p 11 $FMT_I32))
      (set $p (call $make_ascii 16 0x6168735F6C735F5F 0x687375705F776F64)) ;; "__sl_shadow_push"
      (global.set $sh_push_fn (call $reserve_routine $p 16)))
    (when $has_top_vals
      (set $p (call $make_ascii 11 0x696E695F6C735F5F 0x646574)) ;; "__sl_inited"
      (global.set $init_flag_glob (call $ensure_global_vt $p 11 $FMT_I32))
      (set $p (call $make_ascii 9 0x696E695F6C735F5F 0x74)) ;; "__sl_init"
      (global.set $init_fn_idx (call $reserve_routine $p 9))))

  (func $emit_shadow_push_helper
    (local $alloc_idx i32)
    (local $fb        i32)
    (local $p_name    i32)
    (local $old_sp    i32)
    (local $new_sp    i32)
    (local $body_p    i32)

    (when (i32.eqz $uses_shadow) (return))
    (set $alloc_idx (call $ensure_alloc_helper))
    (set $fb (call $fb_new $FMT_I32))
    (set $p_name (call $make_ascii 5 0x7365747962 0)) ;; "bytes"
    (drop (call $fb_add_param $fb $p_name 5 $FMT_I32))
    (set $old_sp (call $fb_alloc_local $fb $FMT_I32))
    (set $new_sp (call $fb_alloc_local $fb $FMT_I32))

    ;; if (__shadow_base == 0) { alloc 64 KiB }
    (call $fb_op_u32 $fb 0x23 $sh_base_glob)
    (call $fb_op $fb 0x45)
    (call $fb_block $fb 0x04 0)
    (call $fb_i32_const $fb $SHADOW_STACK_BYTES)
    (call $fb_op_u32 $fb 0x10 $alloc_idx)
    (call $fb_op_u32 $fb 0x22 $old_sp)
    (call $fb_op_u32 $fb 0x24 $sh_base_glob)
    (call $fb_op_u32 $fb 0x20 $old_sp)
    (call $fb_op_u32 $fb 0x24 $sh_sp_glob)
    (call $fb_op_u32 $fb 0x20 $old_sp)
    (call $fb_i32_const $fb $SHADOW_STACK_BYTES)
    (call $fb_op $fb 0x6A)
    (call $fb_op_u32 $fb 0x24 $sh_lim_glob)
    (call $fb_op $fb 0x0B)

    ;; old_sp = __shadow_sp; new_sp = old_sp + bytes; if (new_sp > __shadow_limit) unreachable;
    (call $fb_op_u32 $fb 0x23 $sh_sp_glob)
    (call $fb_op_u32 $fb 0x22 $old_sp)
    (call $fb_op_u32 $fb 0x20 0)
    (call $fb_op $fb 0x6A)
    (call $fb_op_u32 $fb 0x22 $new_sp)
    (call $fb_op_u32 $fb 0x23 $sh_lim_glob)
    (call $fb_op $fb 0x4B)
    (call $fb_block $fb 0x04 0)
    (call $fb_op $fb 0x00)
    (call $fb_op $fb 0x0B)

    (call $fb_op_u32 $fb 0x20 $new_sp)
    (call $fb_op_u32 $fb 0x24 $sh_sp_glob)
    (call $fb_op_u32 $fb 0x20 $old_sp)
    (call $fb_i32_const $fb 0)
    (call $fb_mem $fb 0x36 2 0)
    (call $fb_op_u32 $fb 0x20 $old_sp)

    (set $body_p (call $fb_finish $fb))
    (call $define_routine $sh_push_fn $body_p $ret_len))

  (func $link_scalar_op (param $op_cp i32) (param $ty_id i32) (result i32)
    (call $link_builtin $op_cp 1 (call $ty_scalar_fmt_cp $ty_id) 0 0))

  (func $link_seq_op (param $op_cp i32) (param $cont_cp i32) (param $elem_ty_id i32) (result i32)
    (if (call $ty_is_scalar $elem_ty_id)
      (then
        (return (call $link_builtin $op_cp 2 $cont_cp (call $ty_scalar_fmt_cp $elem_ty_id) 0)))
      (else
        (return (call $link_builtin $op_cp 3 $cont_cp $DC_STAGEL_LIST_TYPE $DC_TYPE_ANY))))
    (unreachable))

  (func $emit_load_slot (param $fb i32) (param $stride i32) (param $align_exp i32) (param $offset i32)
    (when (i32.eq $stride 1)
      (call $fb_mem $fb 0x2D 0 $offset)
      (return))
    (when (i32.eq $stride 4)
      (call $fb_mem $fb 0x28 $align_exp $offset)
      (return))
    (when (i32.eq $stride 8)
      (call $fb_mem $fb 0x29 $align_exp $offset)
      (return))
    (unreachable))

  (func $emit_store_slot (param $fb i32) (param $stride i32) (param $align_exp i32) (param $offset i32)
    (when (i32.eq $stride 1)
      (call $fb_mem $fb 0x3A 0 $offset)
      (return))
    (when (i32.eq $stride 4)
      (call $fb_mem $fb 0x36 $align_exp $offset)
      (return))
    (when (i32.eq $stride 8)
      (call $fb_mem $fb 0x37 $align_exp $offset)
      (return))
    (unreachable))

  (func $emit_inline_val_eq (param $fb i32) (param $ty_id i32)
    (local $t     i32)
    (local $kind  i32)
    (local $inner i32)
    (when (call $ty_contains_buf $ty_id) (unreachable))
    (when (call $ty_is_scalar $ty_id)
      (call $emit_scalar_op $fb 2 $DC_EQ_OP $ty_id)
      (return))
    (when (i32.eq $ty_id $TY_TUP0)
      (call $fb_op $fb 0x1A)
      (call $fb_op $fb 0x1A)
      (call $fb_i32_const $fb 1)
      (return))
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (set $kind (i32.load $t))
    (when (i32.eq $kind $TK_LIST)
      (set $inner (i32.load offset=4 $t))
      (when (call $ty_is_scalar $inner)
        (call $fb_op_u32
          $fb 0x10
          (call $link_builtin $DC_EQ_OP 2 $DC_STAGEL_LIST_TYPE (call $ty_scalar_fmt_cp $inner) 0))
        (return)))
    (call $fb_op_u32 $fb 0x10 (call $ensure_structural_eq_helper $ty_id)))

  (func $ensure_structural_eq_helper (param $ty_id i32) (result i32)
    (local $t       i32)
    (local $kind    i32)
    (local $fn_idx  i32)
    (local $nm_p    i32)
    (local $fb      i32)
    (local $pa      i32)
    (local $pb      i32)
    (local $inner   i32)
    (local $stride  i32)
    (local $aexp    i32)
    (local $len_loc i32)
    (local $pa_loc  i32)
    (local $pb_loc  i32)
    (local $i_loc   i32)
    (local $arity   i32)
    (local $lptr    i32)
    (local $i       i32)
    (local $ep      i32)
    (local $body_p  i32)

    (when (call $ty_contains_buf $ty_id) (unreachable))
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (when (i32.gt_u (i32.load offset=24 $t) 0)
      (return (i32.sub (i32.load offset=24 $t) 1)))

    (set $nm_p (call $make_tag_name 101 $ty_id)) ;; "__e_<ty_id>"
    (set $fn_idx (call $reserve_routine $nm_p $ret_len))
    (i32.store offset=24 $t (i32.add $fn_idx 1))

    (set $fb (call $fb_new $FMT_I32))
    (set $pa (call $make_ascii 1 97 0)) ;; "a"
    (set $pb (call $make_ascii 1 98 0)) ;; "b"
    (drop (call $fb_add_param $fb $pa 1 $FMT_I32))
    (drop (call $fb_add_param $fb $pb 1 $FMT_I32))

    ;; if (a == b) return 1;
    (call $fb_op_u32 $fb 0x20 0)
    (call $fb_op_u32 $fb 0x20 1)
    (call $fb_op $fb 0x46)
    (call $fb_block $fb 0x04 0)
    (call $fb_i32_const $fb 1)
    (call $fb_op $fb 0x0F)
    (call $fb_op $fb 0x0B)

    (set $kind (i32.load $t))
    (if (i32.eq $kind $TK_LIST)
      (then
        (set $inner (i32.load offset=4 $t))
        (set $stride (call $ty_stride $inner))
        (set $aexp (call $ty_align_exp $inner))
        (set $len_loc (call $fb_alloc_local $fb $FMT_I32))
        (set $pa_loc (call $fb_alloc_local $fb $FMT_I32))
        (set $pb_loc (call $fb_alloc_local $fb $FMT_I32))
        (set $i_loc (call $fb_alloc_local $fb $FMT_I32))

        (call $fb_op_u32 $fb 0x20 0)
        (call $fb_mem $fb 0x28 2 4)
        (call $fb_op_u32 $fb 0x22 $len_loc)
        (call $fb_op_u32 $fb 0x20 1)
        (call $fb_mem $fb 0x28 2 4)
        (call $fb_op $fb 0x46)
        (call $fb_op $fb 0x45)
        (call $fb_block $fb 0x04 0)
        (call $fb_i32_const $fb 0)
        (call $fb_op $fb 0x0F)
        (call $fb_op $fb 0x0B)

        (call $fb_op_u32 $fb 0x20 0)
        (call $fb_mem $fb 0x28 2 0)
        (call $fb_op_u32 $fb 0x21 $pa_loc)
        (call $fb_op_u32 $fb 0x20 1)
        (call $fb_mem $fb 0x28 2 0)
        (call $fb_op_u32 $fb 0x21 $pb_loc)
        (call $fb_i32_const $fb 0)
        (call $fb_op_u32 $fb 0x21 $i_loc)

        (call $fb_block $fb 0x02 0)
        (call $fb_block $fb 0x03 0)
        (call $fb_op_u32 $fb 0x20 $i_loc)
        (call $fb_op_u32 $fb 0x20 $len_loc)
        (call $fb_op $fb 0x4F)
        (call $fb_op_u32 $fb 0x0D 1)

        (call $fb_op_u32 $fb 0x20 $pa_loc)
        (call $fb_op_u32 $fb 0x20 $i_loc)
        (call $fb_i32_const $fb $stride)
        (call $fb_op $fb 0x6C)
        (call $fb_op $fb 0x6A)
        (call $emit_load_slot $fb $stride $aexp 0)

        (call $fb_op_u32 $fb 0x20 $pb_loc)
        (call $fb_op_u32 $fb 0x20 $i_loc)
        (call $fb_i32_const $fb $stride)
        (call $fb_op $fb 0x6C)
        (call $fb_op $fb 0x6A)
        (call $emit_load_slot $fb $stride $aexp 0)

        (call $emit_inline_val_eq $fb $inner)
        (call $fb_op $fb 0x45)
        (call $fb_block $fb 0x04 0)
        (call $fb_i32_const $fb 0)
        (call $fb_op $fb 0x0F)
        (call $fb_op $fb 0x0B)

        (call $fb_op_u32 $fb 0x20 $i_loc)
        (call $fb_i32_const $fb 1)
        (call $fb_op $fb 0x6A)
        (call $fb_op_u32 $fb 0x21 $i_loc)
        (call $fb_op_u32 $fb 0x0C 0)
        (call $fb_op $fb 0x0B)
        (call $fb_op $fb 0x0B))
      (else
        (call $ensure_composite_layout $ty_id)
        (set $arity
          (if (result i32) (i32.eq $kind $TK_TUPLE)
            (then (i32.load offset=4 $t))
            (else (i32.load offset=32 (call $buf_get_i32 $all_structs (i32.load offset=4 $t))))))
        (set $lptr (i32.load offset=20 $t))
        (set $i 0)
        (block $cdone
          (loop $clp
            (br_if $cdone (i32.ge_u $i $arity))
            (set $ep (i32.add $lptr (i32.shl $i 4)))
            (call $fb_op_u32 $fb 0x20 0)
            (call $emit_load_slot $fb (i32.load offset=4 $ep) (i32.load offset=8 $ep) (i32.load $ep))
            (call $fb_op_u32 $fb 0x20 1)
            (call $emit_load_slot $fb (i32.load offset=4 $ep) (i32.load offset=8 $ep) (i32.load $ep))
            (call $emit_inline_val_eq $fb (i32.load offset=12 $ep))
            (call $fb_op $fb 0x45)
            (call $fb_block $fb 0x04 0)
            (call $fb_i32_const $fb 0)
            (call $fb_op $fb 0x0F)
            (call $fb_op $fb 0x0B)
            (set $i (i32.add $i 1))
            (br $clp)))))

    (call $fb_i32_const $fb 1)
    (set $body_p (call $fb_finish $fb))
    (call $define_routine $fn_idx $body_p $ret_len)
    (get $fn_idx))


  ;; ---------------------------------------------------------------------------
  ;; 9. Capture Analysis, Expression & Routine Compiler, and `step` Entrypoint
  ;; ---------------------------------------------------------------------------

  ;; FuncEmitCtx (48 bytes):
  ;; +0 mod_idx, +4 routine_uid, +8 is_nested, +12 ret_ty_id, +16 entry_sp_loc,
  ;; +20 caller_restore_sp_loc, +24 fb, +28 scopes_buf, +32 loops_buf,
  ;; +36 control_depth, +40 is_const_init
  ;; ScopeLayer (16 bytes): +0 vars_buf (VarBinding*), +4 routs_buf (NestedEntry*)
  ;; VarBinding (32 bytes): +0 n_p, +4 n_l, +8 is_cell, +12 loc_idx, +16 ty_id, +20 is_mut, +24 init_flag_loc
  ;; CaptureSpec (16 bytes): +0 n_p, +4 n_l, +8 ty_id, +12 is_cell
  ;; NestedEntry (32 bytes): +0 n_p, +4 n_l, +8 func_idx, +12 decl_uid, +16 rout_ast, +20 caps_buf

  (func $alloc_routine_uid (result i32)
    (local $u i32)
    (set $u $next_rout_uid)
    (global.set $next_rout_uid (i32.add $u 1))
    (get $u))

  (func $lookup_var (param $fctx i32) (param $n_p i32) (param $n_l i32) (result i32)
    (local $sb   i32)
    (local $scnt i32)
    (local $si   i32)
    (local $sl   i32)
    (local $vb   i32)
    (local $vcnt i32)
    (local $vi   i32)
    (local $v    i32)
    (set $sb (i32.load offset=28 $fctx))
    (set $scnt (call $buf_count_i32 $sb))
    (set $si $scnt)
    (block $sdone
      (loop $slp
        (br_if $sdone (i32.eqz $si))
        (set $si (i32.sub $si 1))
        (set $sl (call $buf_get_i32 $sb $si))
        (set $vb (i32.load $sl))
        (set $vcnt (call $buf_count_i32 $vb))
        (set $vi $vcnt)
        (block $vdone
          (loop $vlp
            (br_if $vdone (i32.eqz $vi))
            (set $vi (i32.sub $vi 1))
            (set $v (call $buf_get_i32 $vb $vi))
            (when (call $slice_eq (i32.load $v) (i32.load offset=4 $v) $n_p $n_l)
              (return $v))
            (br $vlp)))
        (br $slp)))
    (i32.const 0))

  (func $lookup_nested_rout (param $fctx i32) (param $n_p i32) (param $n_l i32) (result i32)
    (local $sb   i32)
    (local $scnt i32)
    (local $si   i32)
    (local $sl   i32)
    (local $rb   i32)
    (local $rcnt i32)
    (local $ri   i32)
    (local $nr   i32)
    (set $sb (i32.load offset=28 $fctx))
    (set $scnt (call $buf_count_i32 $sb))
    (set $si $scnt)
    (block $sdone
      (loop $slp
        (br_if $sdone (i32.eqz $si))
        (set $si (i32.sub $si 1))
        (set $sl (call $buf_get_i32 $sb $si))
        (set $rb (i32.load offset=4 $sl))
        (set $rcnt (call $buf_count_i32 $rb))
        (set $ri $rcnt)
        (block $rdone
          (loop $rlp
            (br_if $rdone (i32.eqz $ri))
            (set $ri (i32.sub $ri 1))
            (set $nr (call $buf_get_i32 $rb $ri))
            (when (call $slice_eq (i32.load $nr) (i32.load offset=4 $nr) $n_p $n_l)
              (return $nr))
            (br $rlp)))
        (br $slp)))
    (i32.const 0))

  ;; NameList helper: `$buf` of `NameSlice*` (`+0 ptr, +4 len`)
  (func $nlist_contains (param $nl i32) (param $p i32) (param $l i32) (result i32)
    (local $cnt i32)
    (local $i   i32)
    (local $ns  i32)
    (set $cnt (call $buf_count_i32 $nl))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $ns (call $buf_get_i32 $nl $i))
        (when (call $slice_eq (i32.load $ns) (i32.load offset=4 $ns) $p $l)
          (return 1))
        (set $i (i32.add $i 1))
        (br $lp)))
    (i32.const 0))

  (func $nlist_insert (param $nl i32) (param $p i32) (param $l i32) (result i32)
    (local $ns i32)
    (when (call $nlist_contains $nl $p $l) (return 0))
    (set $ns (call $alloc 8))
    (i32.store $ns $p)
    (i32.store offset=4 $ns $l)
    (call $buf_push_i32 $nl $ns)
    (i32.const 1))

  (func $nlist_clone (param $nl i32) (result i32)
    (local $out i32)
    (local $cnt i32)
    (local $i   i32)
    (set $cnt (call $buf_count_i32 $nl))
    (set $out (call $buf_new (i32.add 16 (i32.shl $cnt 2))))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (call $buf_push_i32 $out (call $buf_get_i32 $nl $i))
        (set $i (i32.add $i 1))
        (br $lp)))
    (get $out))

  ;; Collects free variable names & called nested routine names inside `$ex`.
  (func $collect_routine_refs (param $ex i32) (param $bound i32) (param $out_vars i32) (param $out_calls i32)
    (local $tag   i32)
    (local $elems i32)
    (local $cnt   i32)
    (local $i     i32)
    (local $sub   i32)
    (local $ibnd  i32)
    (local $rbnd  i32)
    (local $r     i32)
    (local $pb    i32)
    (local $pcnt  i32)
    (local $pi    i32)
    (local $prm   i32)

    (when (i32.eqz $ex) (return))
    (set $tag (i32.load $ex))

    (when (i32.eq $tag $EX_VALUE_REF)
      (when (i32.eqz (i32.load offset=8 $ex))
        (when (i32.eqz (call $nlist_contains $bound (i32.load offset=12 $ex) (i32.load offset=16 $ex)))
          (drop (call $nlist_insert $out_vars (i32.load offset=12 $ex) (i32.load offset=16 $ex)))))
      (return))

    (when (i32.eq $tag $EX_SET)
      (when (i32.eqz (call $nlist_contains $bound (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
        (drop (call $nlist_insert $out_vars (i32.load offset=8 $ex) (i32.load offset=12 $ex))))
      (call $collect_routine_refs (i32.load offset=16 $ex) $bound $out_vars $out_calls)
      (return))

    (when (i32.eq $tag $EX_CALL)
      (when (i32.eqz (i32.load offset=4 $ex))
        (drop (call $nlist_insert $out_calls (i32.load offset=8 $ex) (i32.load offset=12 $ex))))
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $cdone
        (loop $clp
          (br_if $cdone (i32.ge_u $i $cnt))
          (call $collect_routine_refs (call $buf_get_i32 $elems $i) $bound $out_vars $out_calls)
          (set $i (i32.add $i 1))
          (br $clp)))
      (return))

    (when (i32.or (i32.eq $tag $EX_LIST) (i32.eq $tag $EX_STRUCT_NEW))
      (set $elems
        (if (result i32) (i32.eq $tag $EX_LIST)
          (then (i32.load offset=8 $ex))
          (else (i32.load offset=16 $ex))))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $ldone
        (loop $llp
          (br_if $ldone (i32.ge_u $i $cnt))
          (call $collect_routine_refs (call $buf_get_i32 $elems $i) $bound $out_vars $out_calls)
          (set $i (i32.add $i 1))
          (br $llp)))
      (return))

    (when (i32.eq $tag $EX_TUPLE_NEW)
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $cnt))
          (call $collect_routine_refs (call $buf_get_i32 $elems $i) $bound $out_vars $out_calls)
          (set $i (i32.add $i 1))
          (br $tlp)))
      (return))

    (when (i32.eq $tag $EX_BLOCK)
      (set $ibnd (call $nlist_clone $bound))
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $bdone
        (loop $blp
          (br_if $bdone (i32.ge_u $i $cnt))
          (set $sub (call $buf_get_i32 $elems $i))
          (if (i32.or (i32.eq (i32.load $sub) $EX_LET) (i32.eq (i32.load $sub) $EX_CONST))
            (then
              (call $collect_routine_refs (i32.load offset=16 $sub) $ibnd $out_vars $out_calls)
              (drop (call $nlist_insert $ibnd (i32.load offset=8 $sub) (i32.load offset=12 $sub))))
            (else
              (call $collect_routine_refs $sub $ibnd $out_vars $out_calls)))
          (set $i (i32.add $i 1))
          (br $blp)))
      (return))

    (when (i32.eq $tag $EX_ROUTINE)
      (set $r (i32.load offset=4 $ex))
      (set $rbnd (call $nlist_clone $bound))
      (set $pb (i32.load offset=24 $r))
      (set $pcnt (call $buf_count_i32 $pb))
      (set $pi 0)
      (block $pdone
        (loop $plp
          (br_if $pdone (i32.ge_u $pi $pcnt))
          (set $prm (call $buf_get_i32 $pb $pi))
          (drop (call $nlist_insert $rbnd (i32.load offset=4 $prm) (i32.load offset=8 $prm)))
          (set $pi (i32.add $pi 1))
          (br $plp)))
      (call $collect_routine_refs (i32.load offset=28 $r) $rbnd $out_vars $out_calls)
      (return))

    (when (i32.eq $tag $EX_FOREACH)
      (call $collect_routine_refs (i32.load offset=16 $ex) $bound $out_vars $out_calls)
      (set $ibnd (call $nlist_clone $bound))
      (drop (call $nlist_insert $ibnd (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
      (call $collect_routine_refs (i32.load offset=20 $ex) $ibnd $out_vars $out_calls)
      (return))

    (when (i32.or (i32.eq $tag $EX_FIELD) (i32.or (i32.eq $tag $EX_CONV) (i32.eq $tag $EX_BITCAST)))
      (call $collect_routine_refs (i32.load offset=4 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.or (i32.eq $tag $EX_BUF_NEW) (i32.eq $tag $EX_UNARY))
      (call $collect_routine_refs (i32.load offset=8 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.eq $tag $EX_RETURN)
      (call $collect_routine_refs (i32.load offset=4 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.eq $tag $EX_WITH)
      (call $collect_routine_refs (i32.load offset=4 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=12 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.eq $tag $EX_WHILE)
      (call $collect_routine_refs (i32.load offset=4 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=8 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.eq $tag $EX_BINARY)
      (call $collect_routine_refs (i32.load offset=8 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=12 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.eq $tag $EX_IF)
      (call $collect_routine_refs (i32.load offset=4 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=8 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=12 $ex) $bound $out_vars $out_calls)
      (return))
    (when (i32.eq $tag $EX_TERNARY)
      (call $collect_routine_refs (i32.load offset=8 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=12 $ex) $bound $out_vars $out_calls)
      (call $collect_routine_refs (i32.load offset=16 $ex) $bound $out_vars $out_calls)
      (return)))

  ;; Checks if `$ex` contains `EX_SET` targeting unshadowed `($n_p, $n_l)`.
  (func $expr_sets_unshadowed (param $ex i32) (param $n_p i32) (param $n_l i32) (result i32)
    (local $tag   i32)
    (local $elems i32)
    (local $cnt   i32)
    (local $i     i32)
    (local $sub   i32)
    (local $r     i32)
    (local $pb    i32)
    (local $pcnt  i32)
    (local $prm   i32)

    (when (i32.eqz $ex) (return 0))
    (set $tag (i32.load $ex))

    (when (i32.eq $tag $EX_SET)
      (when (call $slice_eq (i32.load offset=8 $ex) (i32.load offset=12 $ex) $n_p $n_l)
        (return 1))
      (return (call $expr_sets_unshadowed (i32.load offset=16 $ex) $n_p $n_l)))

    (when (i32.eq $tag $EX_BLOCK)
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $bdone
        (loop $blp
          (br_if $bdone (i32.ge_u $i $cnt))
          (set $sub (call $buf_get_i32 $elems $i))
          (if (i32.or (i32.eq (i32.load $sub) $EX_LET) (i32.eq (i32.load $sub) $EX_CONST))
            (then
              (when (call $expr_sets_unshadowed (i32.load offset=16 $sub) $n_p $n_l)
                (return 1))
              (when (call $slice_eq (i32.load offset=8 $sub) (i32.load offset=12 $sub) $n_p $n_l)
                (return 0)))
            (else
              (when (call $expr_sets_unshadowed $sub $n_p $n_l) (return 1))))
          (set $i (i32.add $i 1))
          (br $blp)))
      (return 0))

    (when (i32.eq $tag $EX_ROUTINE)
      (set $r (i32.load offset=4 $ex))
      (set $pb (i32.load offset=24 $r))
      (set $pcnt (call $buf_count_i32 $pb))
      (set $i 0)
      (block $pdone
        (loop $plp
          (br_if $pdone (i32.ge_u $i $pcnt))
          (set $prm (call $buf_get_i32 $pb $i))
          (when (call $slice_eq (i32.load offset=4 $prm) (i32.load offset=8 $prm) $n_p $n_l)
            (return 0))
          (set $i (i32.add $i 1))
          (br $plp)))
      (return (call $expr_sets_unshadowed (i32.load offset=28 $r) $n_p $n_l)))

    (when (i32.eq $tag $EX_FOREACH)
      (when (call $expr_sets_unshadowed (i32.load offset=16 $ex) $n_p $n_l) (return 1))
      (when (call $slice_eq (i32.load offset=8 $ex) (i32.load offset=12 $ex) $n_p $n_l)
        (return 0))
      (return (call $expr_sets_unshadowed (i32.load offset=20 $ex) $n_p $n_l)))

    (when (i32.or (i32.eq $tag $EX_CALL) (i32.eq $tag $EX_STRUCT_NEW))
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $cdone
        (loop $clp
          (br_if $cdone (i32.ge_u $i $cnt))
          (when (call $expr_sets_unshadowed (call $buf_get_i32 $elems $i) $n_p $n_l) (return 1))
          (set $i (i32.add $i 1))
          (br $clp)))
      (return 0))

    (when (i32.or (i32.eq $tag $EX_LIST) (i32.eq $tag $EX_TUPLE_NEW))
      (set $elems (if (result i32) (i32.eq $tag $EX_LIST) (then (i32.load offset=8 $ex)) (else (i32.load offset=4 $ex))))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $ldone
        (loop $llp
          (br_if $ldone (i32.ge_u $i $cnt))
          (when (call $expr_sets_unshadowed (call $buf_get_i32 $elems $i) $n_p $n_l) (return 1))
          (set $i (i32.add $i 1))
          (br $llp)))
      (return 0))

    (when (i32.or (i32.eq $tag $EX_FIELD) (i32.or (i32.eq $tag $EX_CONV) (i32.or (i32.eq $tag $EX_BITCAST) (i32.eq $tag $EX_RETURN))))
      (return (call $expr_sets_unshadowed (i32.load offset=4 $ex) $n_p $n_l)))
    (when (i32.or (i32.eq $tag $EX_BUF_NEW) (i32.eq $tag $EX_UNARY))
      (return (call $expr_sets_unshadowed (i32.load offset=8 $ex) $n_p $n_l)))
    (when (i32.eq $tag $EX_WITH)
      (return (i32.or (call $expr_sets_unshadowed (i32.load offset=4 $ex) $n_p $n_l) (call $expr_sets_unshadowed (i32.load offset=12 $ex) $n_p $n_l))))
    (when (i32.eq $tag $EX_WHILE)
      (return (i32.or (call $expr_sets_unshadowed (i32.load offset=4 $ex) $n_p $n_l) (call $expr_sets_unshadowed (i32.load offset=8 $ex) $n_p $n_l))))
    (when (i32.eq $tag $EX_BINARY)
      (return (i32.or (call $expr_sets_unshadowed (i32.load offset=8 $ex) $n_p $n_l) (call $expr_sets_unshadowed (i32.load offset=12 $ex) $n_p $n_l))))
    (when (i32.eq $tag $EX_IF)
      (return
        (i32.or
          (call $expr_sets_unshadowed (i32.load offset=4 $ex) $n_p $n_l)
          (i32.or
            (call $expr_sets_unshadowed (i32.load offset=8 $ex) $n_p $n_l)
            (call $expr_sets_unshadowed (i32.load offset=12 $ex) $n_p $n_l)))))
    (when (i32.eq $tag $EX_TERNARY)
      (return
        (i32.or
          (call $expr_sets_unshadowed (i32.load offset=8 $ex) $n_p $n_l)
          (i32.or
            (call $expr_sets_unshadowed (i32.load offset=12 $ex) $n_p $n_l)
            (call $expr_sets_unshadowed (i32.load offset=16 $ex) $n_p $n_l)))))
    (i32.const 0))

  ;; Collects unshadowed references from descendant `EX_ROUTINE`s inside non-routine `$ex`.
  (func $collect_descendant_rout_refs (param $fctx i32) (param $ex i32) (param $bound i32) (param $sib_names i32) (param $sib_caps i32) (param $out_vars i32)
    (local $tag   i32)
    (local $elems i32)
    (local $cnt   i32)
    (local $i     i32)
    (local $j     i32)
    (local $sub   i32)
    (local $ibnd  i32)
    (local $rbnd  i32)
    (local $r     i32)
    (local $pb    i32)
    (local $pcnt  i32)
    (local $prm   i32)
    (local $calls i32)
    (local $ccnt  i32)
    (local $cns   i32)
    (local $scnt  i32)
    (local $sns   i32)
    (local $clst  i32)
    (local $vcnt  i32)
    (local $vns   i32)
    (local $enc   i32)
    (local $ecaps i32)
    (local $cspec i32)

    (when (i32.eqz $ex) (return))
    (set $tag (i32.load $ex))

    (when (i32.eq $tag $EX_ROUTINE)
      (set $r (i32.load offset=4 $ex))
      (set $rbnd (call $nlist_clone $bound))
      (set $pb (i32.load offset=24 $r))
      (set $pcnt (call $buf_count_i32 $pb))
      (set $i 0)
      (block $pdone
        (loop $plp
          (br_if $pdone (i32.ge_u $i $pcnt))
          (set $prm (call $buf_get_i32 $pb $i))
          (drop (call $nlist_insert $rbnd (i32.load offset=4 $prm) (i32.load offset=8 $prm)))
          (set $i (i32.add $i 1))
          (br $plp)))
      (set $calls (call $buf_new 16))
      (call $collect_routine_refs (i32.load offset=28 $r) $rbnd $out_vars $calls)
      (set $ccnt (call $buf_count_i32 $calls))
      (set $i 0)
      (block $cdone
        (loop $clp
          (br_if $cdone (i32.ge_u $i $ccnt))
          (set $cns (call $buf_get_i32 $calls $i))
          (set $scnt (call $buf_count_i32 $sib_names))
          (set $j 0)
          (block $sm_done
            (loop $sm_lp
              (br_if $sm_done (i32.ge_u $j $scnt))
              (set $sns (call $buf_get_i32 $sib_names $j))
              (when (call $slice_eq (i32.load $cns) (i32.load offset=4 $cns) (i32.load $sns) (i32.load offset=4 $sns))
                (set $clst (call $buf_get_i32 $sib_caps $j))
                (set $vcnt (call $buf_count_i32 $clst))
                (set $j 0)
                (block $v_done
                  (loop $v_lp
                    (br_if $v_done (i32.ge_u $j $vcnt))
                    (set $vns (call $buf_get_i32 $clst $j))
                    (when (i32.eqz (call $nlist_contains $rbnd (i32.load $vns) (i32.load offset=4 $vns)))
                      (drop (call $nlist_insert $out_vars (i32.load $vns) (i32.load offset=4 $vns))))
                    (set $j (i32.add $j 1))
                    (br $v_lp)))
                (br $sm_done))
              (set $j (i32.add $j 1))
              (br $sm_lp)))
          (set $enc (call $lookup_nested_rout $fctx (i32.load $cns) (i32.load offset=4 $cns)))
          (when $enc
            (set $ecaps (i32.load offset=20 $enc))
            (set $vcnt (call $buf_count_i32 $ecaps))
            (set $j 0)
            (block $ec_done
              (loop $ec_lp
                (br_if $ec_done (i32.ge_u $j $vcnt))
                (set $cspec (call $buf_get_i32 $ecaps $j))
                (when (i32.eqz (call $nlist_contains $rbnd (i32.load $cspec) (i32.load offset=4 $cspec)))
                  (drop (call $nlist_insert $out_vars (i32.load $cspec) (i32.load offset=4 $cspec))))
                (set $j (i32.add $j 1))
                (br $ec_lp))))
          (set $i (i32.add $i 1))
          (br $clp)))
      (return))

    (when (i32.eq $tag $EX_BLOCK)
      (set $ibnd (call $nlist_clone $bound))
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $bdone
        (loop $blp
          (br_if $bdone (i32.ge_u $i $cnt))
          (set $sub (call $buf_get_i32 $elems $i))
          (if (i32.or (i32.eq (i32.load $sub) $EX_LET) (i32.eq (i32.load $sub) $EX_CONST))
            (then
              (call $collect_descendant_rout_refs $fctx (i32.load offset=16 $sub) $ibnd $sib_names $sib_caps $out_vars)
              (drop (call $nlist_insert $ibnd (i32.load offset=8 $sub) (i32.load offset=12 $sub))))
            (else
              (call $collect_descendant_rout_refs $fctx $sub $ibnd $sib_names $sib_caps $out_vars)))
          (set $i (i32.add $i 1))
          (br $blp)))
      (return))

    (when (i32.eq $tag $EX_FOREACH)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=16 $ex) $bound $sib_names $sib_caps $out_vars)
      (set $ibnd (call $nlist_clone $bound))
      (drop (call $nlist_insert $ibnd (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
      (call $collect_descendant_rout_refs $fctx (i32.load offset=20 $ex) $ibnd $sib_names $sib_caps $out_vars)
      (return))

    (when (i32.or (i32.eq $tag $EX_CALL) (i32.eq $tag $EX_STRUCT_NEW))
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $cadone
        (loop $calp
          (br_if $cadone (i32.ge_u $i $cnt))
          (call $collect_descendant_rout_refs $fctx (call $buf_get_i32 $elems $i) $bound $sib_names $sib_caps $out_vars)
          (set $i (i32.add $i 1))
          (br $calp)))
      (return))

    (when (i32.or (i32.eq $tag $EX_LIST) (i32.eq $tag $EX_TUPLE_NEW))
      (set $elems (if (result i32) (i32.eq $tag $EX_LIST) (then (i32.load offset=8 $ex)) (else (i32.load offset=4 $ex))))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $ltdone
        (loop $ltlp
          (br_if $ltdone (i32.ge_u $i $cnt))
          (call $collect_descendant_rout_refs $fctx (call $buf_get_i32 $elems $i) $bound $sib_names $sib_caps $out_vars)
          (set $i (i32.add $i 1))
          (br $ltlp)))
      (return))

    (when (i32.or (i32.eq $tag $EX_LET) (i32.or (i32.eq $tag $EX_CONST) (i32.eq $tag $EX_SET)))
      (call $collect_descendant_rout_refs $fctx (i32.load offset=16 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.or (i32.eq $tag $EX_FIELD) (i32.or (i32.eq $tag $EX_CONV) (i32.or (i32.eq $tag $EX_BITCAST) (i32.eq $tag $EX_RETURN))))
      (call $collect_descendant_rout_refs $fctx (i32.load offset=4 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.or (i32.eq $tag $EX_BUF_NEW) (i32.eq $tag $EX_UNARY))
      (call $collect_descendant_rout_refs $fctx (i32.load offset=8 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.eq $tag $EX_WITH)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=4 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=12 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.eq $tag $EX_WHILE)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=4 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=8 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.eq $tag $EX_BINARY)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=8 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=12 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.eq $tag $EX_IF)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=4 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=8 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=12 $ex) $bound $sib_names $sib_caps $out_vars)
      (return))
    (when (i32.eq $tag $EX_TERNARY)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=8 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=12 $ex) $bound $sib_names $sib_caps $out_vars)
      (call $collect_descendant_rout_refs $fctx (i32.load offset=16 $ex) $bound $sib_names $sib_caps $out_vars)
      (return)))

  ;; Static expression type inference (for `EX_IF` branch unification & `EX_CALL` overload resolution).
  (func $infer_expr_type (param $fctx i32) (param $ex i32) (result i32)
    (local $tag    i32)
    (local $mod_i  i32)
    (local $elems  i32)
    (local $cnt    i32)
    (local $i      i32)
    (local $cptr   i32)
    (local $ety    i32)
    (local $sidx   i32)
    (local $bty    i32)
    (local $t      i32)
    (local $lptr   i32)
    (local $idx    i32)
    (local $v      i32)
    (local $opt_ty i32)
    (local $tv     i32)
    (local $nr     i32)
    (local $r      i32)
    (local $sb     i32)
    (local $sl     i32)
    (local $vb     i32)
    (local $sub    i32)
    (local $op     i32)
    (local $t1     i32)
    (local $t2     i32)

    (set $tag (i32.load $ex))
    (set $mod_i (i32.load $fctx))

    (when (i32.eq $tag $EX_BOOL)  (return $TY_B))
    (when (i32.eq $tag $EX_INT)   (return (i32.load offset=4 $ex)))
    (when (i32.eq $tag $EX_BYTES) (return (call $intern_unary_ty $TK_LIST $TY_U8)))
    (when (i32.eq $tag $EX_LIST)
      (return (call $intern_unary_ty $TK_LIST (call $canon_type $mod_i (i32.load offset=4 $ex)))))
    (when (i32.eq $tag $EX_TUPLE_NEW)
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (when (i32.eqz $cnt) (return $TY_TUP0))
      (set $cptr (call $alloc (i32.shl $cnt 2)))
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $cnt))
          (set $ety (call $infer_expr_type $fctx (call $buf_get_i32 $elems $i)))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (i32.store (i32.add $cptr (i32.shl $i 2)) $ety)
          (set $i (i32.add $i 1))
          (br $tlp)))
      (return (call $intern_tuple_ty $cnt $cptr)))
    (when (i32.eq $tag $EX_STRUCT_NEW)
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $sndone
        (loop $snlp
          (br_if $sndone (i32.ge_u $i $cnt))
          (when (i32.eq (call $infer_expr_type $fctx (call $buf_get_i32 $elems $i)) $TY_NEVER)
            (return $TY_NEVER))
          (set $i (i32.add $i 1))
          (br $snlp)))
      (set $sidx
        (if (result i32) (i32.load offset=4 $ex)
          (then (call $resolve_struct_dc $mod_i (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
          (else (call $resolve_struct_ident $mod_i (i32.load offset=8 $ex) (i32.load offset=12 $ex)))))
      (when (i32.lt_s $sidx 0) (unreachable))
      (return (call $intern_struct_ty $sidx)))
    (when (i32.eq $tag $EX_FIELD)
      (set $bty (call $infer_expr_type $fctx (i32.load offset=4 $ex)))
      (when (i32.eq $bty $TY_NEVER) (return $TY_NEVER))
      (call $ensure_composite_layout $bty)
      (set $t (call $buf_get_i32 $type_table $bty))
      (set $cnt
        (if (result i32) (i32.eq (i32.load $t) $TK_TUPLE)
          (then (i32.load offset=4 $t))
          (else (i32.load offset=32 (call $buf_get_i32 $all_structs (i32.load offset=4 $t))))))
      (set $idx (i32.load offset=8 $ex))
      (when (i32.ge_u $idx $cnt) (unreachable))
      (set $lptr (i32.load offset=20 $t))
      (return (i32.load offset=12 (i32.add $lptr (i32.shl $idx 4)))))
    (when (i32.eq $tag $EX_WITH)
      (set $bty (call $infer_expr_type $fctx (i32.load offset=4 $ex)))
      (when (i32.eq $bty $TY_NEVER) (return $TY_NEVER))
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=12 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (return $bty))
    (when (i32.eq $tag $EX_BUF_NEW)
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=8 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (return (call $intern_unary_ty $TK_BUF (call $canon_type $mod_i (i32.load offset=4 $ex)))))
    (when (i32.eq $tag $EX_VALUE_REF)
      (set $opt_ty 0)
      (when (i32.load offset=4 $ex)
        (set $opt_ty (i32.add 1 (call $canon_type $mod_i (i32.load offset=4 $ex)))))
      (when (i32.eqz (i32.load offset=8 $ex))
        (set $v (call $lookup_var $fctx (i32.load offset=12 $ex) (i32.load offset=16 $ex)))
        (when $v
          (when (i32.and $opt_ty (i32.ne (i32.sub $opt_ty 1) (i32.load offset=16 $v)))
            (unreachable))
          (return (i32.load offset=16 $v))))
      (set $tv
        (call $resolve_top_val
          $mod_i $opt_ty
          (i32.load offset=8 $ex) (i32.load offset=12 $ex) (i32.load offset=16 $ex)))
      (return (i32.load offset=28 $tv)))
    (when (i32.eq $tag $EX_LET)
      (when (i32.load offset=16 $ex)
        (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=16 $ex)) $TY_NEVER)
          (return $TY_NEVER)))
      (return $TY_VOID))
    (when (i32.or (i32.eq $tag $EX_CONST) (i32.eq $tag $EX_SET))
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=16 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (return $TY_VOID))
    (when (i32.or (i32.eq $tag $EX_ROUTINE) (i32.eq $tag $EX_NOP))
      (return $TY_VOID))
    (when (i32.or (i32.eq $tag $EX_WHILE) (i32.eq $tag $EX_FOREACH))
      (return $TY_VOID))
    (when
      (i32.or
        (i32.or (i32.eq $tag $EX_BREAK) (i32.eq $tag $EX_CONTINUE))
        (i32.or (i32.eq $tag $EX_RETURN) (i32.eq $tag $EX_FAULT)))
      (return $TY_NEVER))
    (when (i32.eq $tag $EX_CALL)
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $cptr (call $alloc (i32.add 4 (i32.shl $cnt 2))))
      (set $i 0)
      (block $cadone
        (loop $calp
          (br_if $cadone (i32.ge_u $i $cnt))
          (set $ety (call $infer_expr_type $fctx (call $buf_get_i32 $elems $i)))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (i32.store (i32.add $cptr (i32.shl $i 2)) $ety)
          (set $i (i32.add $i 1))
          (br $calp)))
      (when (i32.eqz (i32.load offset=4 $ex))
        (set $nr (call $lookup_nested_rout $fctx (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
        (when $nr
          (set $r (i32.load offset=16 $nr))
          (when (i32.eqz (call $routine_matches_args $r $cnt $cptr)) (unreachable))
          (return (i32.load offset=32 $r))))
      (set $r
        (call $resolve_top_rout
          $mod_i
          (i32.load offset=4 $ex) (i32.load offset=8 $ex) (i32.load offset=12 $ex)
          $cnt $cptr))
      (return (i32.load offset=32 $r)))
    (when (i32.eq $tag $EX_BLOCK)
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (when (i32.eqz $cnt) (return $TY_VOID))
      ;; Push temporary inference scope layer
      (set $sb (i32.load offset=28 $fctx))
      (set $sl (call $alloc 16))
      (set $vb (call $buf_new 16))
      (i32.store $sl $vb)
      (i32.store offset=4 $sl (call $buf_new 16))
      ;; Register direct nested routines in temporary inference scope
      (set $i 0)
      (block $nrdone
        (loop $nrlp
          (br_if $nrdone (i32.ge_u $i $cnt))
          (set $sub (call $buf_get_i32 $elems $i))
          (when (i32.eq (i32.load $sub) $EX_ROUTINE)
            (set $r (i32.load offset=4 $sub))
            (i32.store offset=32 $r (call $canon_type $mod_i (i32.load offset=12 $r)))
            (set $t (i32.load offset=24 $r))
            (set $idx 0)
            (block $npdone
              (loop $nplp
                (br_if $npdone (i32.ge_u $idx (call $buf_count_i32 $t)))
                (set $v (call $buf_get_i32 $t $idx))
                (i32.store offset=12 $v (call $canon_type $mod_i (i32.load $v)))
                (set $idx (i32.add $idx 1))
                (br $nplp)))
            (set $nr (call $alloc 32))
            (i32.store $nr (i32.load offset=16 $r))
            (i32.store offset=4 $nr (i32.load offset=20 $r))
            (i32.store offset=16 $nr $r)
            (call $buf_push_i32 (i32.load offset=4 $sl) $nr))
          (set $i (i32.add $i 1))
          (br $nrlp)))
      (call $buf_push_i32 $sb $sl)
      (set $ety $TY_VOID)
      (set $i 0)
      (block $bdone
        (loop $blp
          (br_if $bdone (i32.ge_u $i $cnt))
          (set $sub (call $buf_get_i32 $elems $i))
          (set $ety (call $infer_expr_type $fctx $sub))
          (when (i32.or (i32.eq (i32.load $sub) $EX_LET) (i32.eq (i32.load $sub) $EX_CONST))
            (set $v (call $alloc 32))
            (i32.store $v (i32.load offset=8 $sub))
            (i32.store offset=4 $v (i32.load offset=12 $sub))
            (i32.store offset=16 $v (call $canon_type $mod_i (i32.load offset=4 $sub)))
            (call $buf_push_i32 $vb $v))
          (when (i32.eq $ety $TY_NEVER)
            (i32.store offset=4 $sb (i32.sub (i32.load offset=4 $sb) 4))
            (return $TY_NEVER))
          (set $i (i32.add $i 1))
          (br $blp)))
      (i32.store offset=4 $sb (i32.sub (i32.load offset=4 $sb) 4))
      (return $ety))
    (when (i32.eq $tag $EX_IF)
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=4 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (set $t1 (call $infer_expr_type $fctx (i32.load offset=8 $ex)))
      (set $t2 (call $infer_expr_type $fctx (i32.load offset=12 $ex)))
      (when (i32.eq $t1 $TY_NEVER) (return $t2))
      (when (i32.eq $t2 $TY_NEVER) (return $t1))
      (when (i32.ne $t1 $t2) (unreachable))
      (return $t1))
    (when (i32.eq $tag $EX_UNARY)
      (set $op (i32.load offset=4 $ex))
      (set $t1 (call $infer_expr_type $fctx (i32.load offset=8 $ex)))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (when (i32.eq $op $DC_LOGICAL_NOT_OP) (return $TY_B))
      (when (i32.and (i32.ge_u $op $DC_CLZ_OP) (i32.le_u $op $DC_POPCNT_OP)) (return $TY_U32))
      (when (i32.eq $op $DC_SEQ_LEN_OP) (return $TY_U64))
      (when (i32.eq $op $DC_STAGEL_BUF_FREEZE)
        (set $t (call $buf_get_i32 $type_table $t1))
        (when (i32.ne (i32.load $t) $TK_BUF) (unreachable))
        (return (call $intern_unary_ty $TK_LIST (i32.load offset=4 $t))))
      (when (i32.eq $op $DC_REQUEST_HOST_CAPABILITY_OP)
        (when (i32.lt_u $t1 8) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when
          (i32.or
            (i32.ne (i32.load $t) $TK_LIST)
            (i32.ne (i32.load offset=4 $t) $TY_U8))
          (unreachable))
        (return $t1))
      (return $t1))
    (when (i32.eq $tag $EX_BINARY)
      (set $op (i32.load offset=4 $ex))
      (set $t1 (call $infer_expr_type $fctx (i32.load offset=8 $ex)))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (set $t2 (call $infer_expr_type $fctx (i32.load offset=12 $ex)))
      (when (i32.and (i32.eq $t2 $TY_NEVER) (i32.and (i32.ne $op $DC_LOGICAL_AND_OP) (i32.ne $op $DC_LOGICAL_OR_OP)))
        (return $TY_NEVER))
      (when (i32.and (i32.ge_u $op $DC_LOGICAL_AND_OP) (i32.le_u $op $DC_GE_OP))
        (return $TY_B))
      (when (i32.eq $op $DC_SEQ_GET_OP)
        (set $t (call $buf_get_i32 $type_table $t1))
        (return (i32.load offset=4 $t)))
      (when (i32.eq $op $DC_STAGEL_BUF_FILL)
        (return (call $intern_unary_ty $TK_BUF $t1)))
      (when (i32.or (i32.eq $op $DC_STAGEL_BUF_PUSH) (i32.eq $op $DC_STAGEL_BUF_EXTEND))
        (return $TY_VOID))
      (return $t1))
    (when (i32.eq $tag $EX_TERNARY)
      (set $op (i32.load offset=4 $ex))
      (set $t1 (call $infer_expr_type $fctx (i32.load offset=8 $ex)))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=12 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=16 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (when (i32.eq $op $DC_STAGEL_BUF_SET) (return $TY_VOID))
      (return $t1))
    (when (i32.or (i32.eq $tag $EX_CONV) (i32.eq $tag $EX_BITCAST))
      (when (i32.eq (call $infer_expr_type $fctx (i32.load offset=4 $ex)) $TY_NEVER)
        (return $TY_NEVER))
      (return (call $canon_type $mod_i (i32.load offset=8 $ex))))
    (unreachable))

  (func $emit_routine_shadow_pop (param $fctx i32)
    (local $fb  i32)
    (local $loc i32)
    (when (i32.eqz $uses_shadow) (return))
    (set $fb (i32.load offset=24 $fctx))
    (set $loc
      (if (result i32) (i32.load offset=8 $fctx)
        (then (i32.load offset=20 $fctx))
        (else (i32.load offset=16 $fctx))))
    (when (i32.ge_s $loc 0)
      (call $fb_op_u32 $fb 0x20 $loc)
      (call $fb_op_u32 $fb 0x24 $sh_sp_glob)))

  (func $emit_read_var (param $fctx i32) (param $v i32) (result i32)
    (local $fb   i32)
    (local $loc  i32)
    (local $ty   i32)
    (local $vt   i32)
    (local $floc i32)
    (set $fb (i32.load offset=24 $fctx))
    (set $loc (i32.load offset=12 $v))
    (set $ty (i32.load offset=16 $v))
    (set $vt (call $ty_wasm_vt $ty))
    (if (i32.load offset=8 $v)
      (then
        ;; Shadow cell read: verify `init_flag != 0` at `+0`, then load value at `+8`
        (call $fb_op_u32 $fb 0x20 $loc)
        (call $fb_mem $fb 0x28 2 0)
        (call $fb_op $fb 0x45)
        (call $fb_block $fb 0x04 0)
        (call $fb_op $fb 0x00)
        (call $fb_op $fb 0x0B)
        (call $fb_op_u32 $fb 0x20 $loc)
        (if (i32.eq $vt $FMT_I64)
          (then (call $fb_mem $fb 0x29 3 8))
          (else (call $fb_mem $fb 0x28 2 8))))
      (else
        (set $floc (i32.load offset=24 $v))
        (when (i32.ge_s $floc 0)
          (call $fb_op_u32 $fb 0x20 $floc)
          (call $fb_op $fb 0x45)
          (call $fb_block $fb 0x04 0)
          (call $fb_op $fb 0x00)
          (call $fb_op $fb 0x0B))
        (call $fb_op_u32 $fb 0x20 $loc)))
    (get $ty))

  (func $emit_write_cell (param $fb i32) (param $ptr_loc i32) (param $ty_id i32) (param $val_loc i32)
    (call $fb_op_u32 $fb 0x20 $ptr_loc)
    (call $fb_op_u32 $fb 0x20 $val_loc)
    (if (i32.eq (call $ty_wasm_vt $ty_id) $FMT_I64)
      (then (call $fb_mem $fb 0x37 3 8))
      (else (call $fb_mem $fb 0x36 2 8)))
    (call $fb_op_u32 $fb 0x20 $ptr_loc)
    (call $fb_i32_const $fb 1)
    (call $fb_mem $fb 0x36 2 0))

  (func $emit_store_composite_slots (param $fb i32) (param $ty_id i32) (param $locs_buf i32)
    (local $t         i32)
    (local $tot       i32)
    (local $lptr      i32)
    (local $alloc_idx i32)
    (local $ptr_loc   i32)
    (local $cnt       i32)
    (local $i         i32)
    (local $ep        i32)
    (call $ensure_composite_layout $ty_id)
    (set $t (call $buf_get_i32 $type_table $ty_id))
    (set $tot (i32.load offset=16 $t))
    (set $lptr (i32.load offset=20 $t))
    (set $alloc_idx (call $ensure_alloc_helper))
    (set $ptr_loc (call $fb_alloc_local $fb $FMT_I32))
    (call $fb_i32_const $fb $tot)
    (call $fb_op_u32 $fb 0x10 $alloc_idx)
    (call $fb_op_u32 $fb 0x21 $ptr_loc)
    (set $cnt (call $buf_count_i32 $locs_buf))
    (set $i 0)
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u $i $cnt))
        (set $ep (i32.add $lptr (i32.shl $i 4)))
        (call $fb_op_u32 $fb 0x20 $ptr_loc)
        (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $locs_buf $i))
        (call $emit_store_slot $fb (i32.load offset=4 $ep) (i32.load offset=8 $ep) (i32.load $ep))
        (set $i (i32.add $i 1))
        (br $lp)))
    (call $fb_op_u32 $fb 0x20 $ptr_loc))

  (func $emit_block_expr (param $fctx i32) (param $stmts_buf i32) (param $in_tail i32) (result i32)
    (local $cnt        i32)
    (local $mod_i      i32)
    (local $fb         i32)
    (local $i          i32)
    (local $j          i32)
    (local $k          i32)
    (local $stmt       i32)
    (local $stag       i32)
    (local $n_routs    i32)
    (local $nr_cnt     i32)
    (local $sib_names  i32)
    (local $sib_caps   i32)
    (local $sib_calls  i32)
    (local $r          i32)
    (local $rbnd       i32)
    (local $pb         i32)
    (local $pcnt       i32)
    (local $prm        i32)
    (local $dvars      i32)
    (local $dcalls     i32)
    (local $cns        i32)
    (local $enc        i32)
    (local $ecaps      i32)
    (local $cspec      i32)
    (local $changed    i32)
    (local $clst       i32)
    (local $src_caps   i32)
    (local $vns        i32)
    (local $all_cap    i32)
    (local $slayer     i32)
    (local $svars      i32)
    (local $srouts     i32)
    (local $np         i32)
    (local $nl         i32)
    (local $cty        i32)
    (local $mutated    i32)
    (local $ptr_loc    i32)
    (local $vb         i32)
    (local $vis_names  i32)
    (local $vis_tys    i32)
    (local $vis_cells  i32)
    (local $sb         i32)
    (local $scnt       i32)
    (local $os         i32)
    (local $ovb        i32)
    (local $fn_name    i32)
    (local $fn_idx     i32)
    (local $caps_out   i32)
    (local $nentry     i32)
    (local $inh_scopes i32)
    (local $isl        i32)
    (local $res_ty     i32)
    (local $is_last    i32)

    (set $cnt (call $buf_count_i32 $stmts_buf))
    (when (i32.eqz $cnt) (return $TY_VOID))
    (set $mod_i (i32.load $fctx))
    (set $fb (i32.load offset=24 $fctx))

    ;; 1. Gather direct nested `EX_ROUTINE`s in this block & canonicalize their signatures
    (set $n_routs (call $buf_new 16))
    (set $sib_names (call $buf_new 16))
    (set $sib_caps (call $buf_new 16))
    (set $sib_calls (call $buf_new 16))
    (set $i 0)
    (block $g_done
      (loop $g_lp
        (br_if $g_done (i32.ge_u $i $cnt))
        (set $stmt (call $buf_get_i32 $stmts_buf $i))
        (when (i32.eq (i32.load $stmt) $EX_ROUTINE)
          (when (i32.load offset=40 $fctx) (unreachable))
          (set $r (i32.load offset=4 $stmt))
          (set $cty (call $canon_type $mod_i (i32.load offset=12 $r)))
          (call $validate_type_wf $cty)
          (i32.store offset=32 $r $cty)
          (set $pb (i32.load offset=24 $r))
          (set $pcnt (call $buf_count_i32 $pb))
          (set $j 0)
          (block $gp_done
            (loop $gp_lp
              (br_if $gp_done (i32.ge_u $j $pcnt))
              (set $prm (call $buf_get_i32 $pb $j))
              (set $cty (call $canon_type $mod_i (i32.load $prm)))
              (call $validate_type_wf $cty)
              (when (i32.eqz (call $ty_wasm_vt $cty)) (unreachable))
              (i32.store offset=12 $prm $cty)
              (set $j (i32.add $j 1))
              (br $gp_lp)))
          (when (call $nlist_contains $sib_names (i32.load offset=16 $r) (i32.load offset=20 $r))
            (unreachable))
          (drop (call $nlist_insert $sib_names (i32.load offset=16 $r) (i32.load offset=20 $r)))
          (call $buf_push_i32 $n_routs $r))
        (set $i (i32.add $i 1))
        (br $g_lp)))

    ;; 2. Compute transitive free-variable captures per direct nested routine
    (set $nr_cnt (call $buf_count_i32 $n_routs))
    (set $i 0)
    (block $c1_done
      (loop $c1_lp
        (br_if $c1_done (i32.ge_u $i $nr_cnt))
        (set $r (call $buf_get_i32 $n_routs $i))
        (set $rbnd (call $buf_new 16))
        (set $pb (i32.load offset=24 $r))
        (set $pcnt (call $buf_count_i32 $pb))
        (set $j 0)
        (block $b_done
          (loop $b_lp
            (br_if $b_done (i32.ge_u $j $pcnt))
            (set $prm (call $buf_get_i32 $pb $j))
            (drop (call $nlist_insert $rbnd (i32.load offset=4 $prm) (i32.load offset=8 $prm)))
            (set $j (i32.add $j 1))
            (br $b_lp)))
        (set $dvars (call $buf_new 16))
        (set $dcalls (call $buf_new 16))
        (call $collect_routine_refs (i32.load offset=28 $r) $rbnd $dvars $dcalls)
        ;; Forward captures of called enclosing nested routines
        (set $j 0)
        (block $ec_done
          (loop $ec_lp
            (br_if $ec_done (i32.ge_u $j (call $buf_count_i32 $dcalls)))
            (set $cns (call $buf_get_i32 $dcalls $j))
            (when (i32.eqz (call $nlist_contains $sib_names (i32.load $cns) (i32.load offset=4 $cns)))
              (set $enc (call $lookup_nested_rout $fctx (i32.load $cns) (i32.load offset=4 $cns)))
              (when $enc
                (set $ecaps (i32.load offset=20 $enc))
                (set $k 0)
                (block $ek_done
                  (loop $ek_lp
                    (br_if $ek_done (i32.ge_u $k (call $buf_count_i32 $ecaps)))
                    (set $cspec (call $buf_get_i32 $ecaps $k))
                    (when (i32.eqz (call $nlist_contains $rbnd (i32.load $cspec) (i32.load offset=4 $cspec)))
                      (drop (call $nlist_insert $dvars (i32.load $cspec) (i32.load offset=4 $cspec))))
                    (set $k (i32.add $k 1))
                    (br $ek_lp)))))
            (set $j (i32.add $j 1))
            (br $ec_lp)))
        (call $buf_push_i32 $sib_caps $dvars)
        (call $buf_push_i32 $sib_calls $dcalls)
        (set $i (i32.add $i 1))
        (br $c1_lp)))

    ;; Fixed-point transitive propagation across sibling nested calls
    (block $fp_done
      (loop $fp_lp
        (set $changed 0)
        (set $i 0)
        (block $fpi_done
          (loop $fpi_lp
            (br_if $fpi_done (i32.ge_u $i $nr_cnt))
            (set $dcalls (call $buf_get_i32 $sib_calls $i))
            (set $clst (call $buf_get_i32 $sib_caps $i))
            (set $j 0)
            (block $fpj_done
              (loop $fpj_lp
                (br_if $fpj_done (i32.ge_u $j (call $buf_count_i32 $dcalls)))
                (set $cns (call $buf_get_i32 $dcalls $j))
                (set $k 0)
                (block $fpk_done
                  (loop $fpk_lp
                    (br_if $fpk_done (i32.ge_u $k $nr_cnt))
                    (set $vns (call $buf_get_i32 $sib_names $k))
                    (when (call $slice_eq (i32.load $cns) (i32.load offset=4 $cns) (i32.load $vns) (i32.load offset=4 $vns))
                      (set $src_caps (call $buf_get_i32 $sib_caps $k))
                      (set $pcnt 0)
                      (block $fpx_done
                        (loop $fpx_lp
                          (br_if $fpx_done (i32.ge_u $pcnt (call $buf_count_i32 $src_caps)))
                          (set $prm (call $buf_get_i32 $src_caps $pcnt))
                          (when (call $nlist_insert $clst (i32.load $prm) (i32.load offset=4 $prm))
                            (set $changed 1))
                          (set $pcnt (i32.add $pcnt 1))
                          (br $fpx_lp))))
                    (set $k (i32.add $k 1))
                    (br $fpk_lp)))
                (set $j (i32.add $j 1))
                (br $fpj_lp)))
            (set $i (i32.add $i 1))
            (br $fpi_lp)))
        (br_if $fp_done (i32.eqz $changed))
        (br $fp_lp)))

    ;; Union of all variable names referenced by direct OR descendant nested routines
    (set $all_cap (call $buf_new 16))
    (set $i 0)
    (block $au_done
      (loop $au_lp
        (br_if $au_done (i32.ge_u $i $nr_cnt))
        (set $clst (call $buf_get_i32 $sib_caps $i))
        (set $j 0)
        (block $auj_done
          (loop $auj_lp
            (br_if $auj_done (i32.ge_u $j (call $buf_count_i32 $clst)))
            (set $vns (call $buf_get_i32 $clst $j))
            (drop (call $nlist_insert $all_cap (i32.load $vns) (i32.load offset=4 $vns)))
            (set $j (i32.add $j 1))
            (br $auj_lp)))
        (set $i (i32.add $i 1))
        (br $au_lp)))

    (set $rbnd (call $buf_new 16))
    (set $i 0)
    (block $dr_done
      (loop $dr_lp
        (br_if $dr_done (i32.ge_u $i $cnt))
        (set $stmt (call $buf_get_i32 $stmts_buf $i))
        (when (i32.ne (i32.load $stmt) $EX_ROUTINE)
          (call $collect_descendant_rout_refs $fctx $stmt $rbnd $sib_names $sib_caps $all_cap))
        (set $i (i32.add $i 1))
        (br $dr_lp)))

    ;; 3. Create scope layer & pre-allocate shadow cells for captured mutable/uninit `let`s
    (set $slayer (call $alloc 16))
    (set $svars (call $buf_new 16))
    (set $srouts (call $buf_new 16))
    (i32.store $slayer $svars)
    (i32.store offset=4 $slayer $srouts)

    (set $rbnd (call $buf_new 16)) ;; reject duplicate local var decls in same block
    (set $i 0)
    (block $pre_done
      (loop $pre_lp
        (br_if $pre_done (i32.ge_u $i $cnt))
        (set $stmt (call $buf_get_i32 $stmts_buf $i))
        (set $stag (i32.load $stmt))
        (when (i32.or (i32.eq $stag $EX_LET) (i32.eq $stag $EX_CONST))
          (set $np (i32.load offset=8 $stmt))
          (set $nl (i32.load offset=12 $stmt))
          (when (i32.eqz (call $nlist_insert $rbnd $np $nl)) (unreachable))
          (when (i32.and (i32.eq $stag $EX_LET) (call $nlist_contains $all_cap $np $nl))
            (set $mutated (i32.eqz (i32.load offset=16 $stmt)))
            (when (i32.eqz $mutated)
              (set $j (i32.add $i 1))
              (block $ms_done
                (loop $ms_lp
                  (br_if $ms_done (i32.ge_u $j $cnt))
                  (when (call $expr_sets_unshadowed (call $buf_get_i32 $stmts_buf $j) $np $nl)
                    (set $mutated 1)
                    (br $ms_done))
                  (set $j (i32.add $j 1))
                  (br $ms_lp))))
            (when $mutated
              (set $cty (call $canon_type $mod_i (i32.load offset=4 $stmt)))
              (call $validate_type_wf $cty)
              (when (i32.eqz (call $ty_wasm_vt $cty)) (unreachable))
              (set $ptr_loc (call $fb_alloc_local $fb $FMT_I32))
              (call $fb_i32_const $fb $SHADOW_CELL_BYTES)
              (call $fb_op_u32 $fb 0x10 $sh_push_fn)
              (call $fb_op_u32 $fb 0x21 $ptr_loc)
              (set $vb (call $alloc 32))
              (i32.store $vb $np)
              (i32.store offset=4 $vb $nl)
              (i32.store offset=8 $vb 1)
              (i32.store offset=12 $vb $ptr_loc)
              (i32.store offset=16 $vb $cty)
              (i32.store offset=20 $vb 1)
              (i32.store offset=24 $vb -1)
              (call $buf_push_i32 $svars $vb))))
        (set $i (i32.add $i 1))
        (br $pre_lp)))

    ;; 4. Build visible lexical variable table (this block's decls shadow enclosing scopes)
    (set $vis_names (call $buf_new 16))
    (set $vis_tys (call $buf_new 16))
    (set $vis_cells (call $buf_new 16))
    (when (i32.gt_u $nr_cnt 0)
      (set $i 0)
      (block $vl_done
        (loop $vl_lp
          (br_if $vl_done (i32.ge_u $i $cnt))
          (set $stmt (call $buf_get_i32 $stmts_buf $i))
          (set $stag (i32.load $stmt))
          (when (i32.or (i32.eq $stag $EX_LET) (i32.eq $stag $EX_CONST))
            (set $np (i32.load offset=8 $stmt))
            (set $nl (i32.load offset=12 $stmt))
            (when (call $nlist_insert $vis_names $np $nl)
              (set $cty (call $canon_type $mod_i (i32.load offset=4 $stmt)))
              (call $buf_push_i32 $vis_tys $cty)
              (set $mutated 0)
              (set $j 0)
              (block $fc_done
                (loop $fc_lp
                  (br_if $fc_done (i32.ge_u $j (call $buf_count_i32 $svars)))
                  (set $vb (call $buf_get_i32 $svars $j))
                  (when (call $slice_eq (i32.load $vb) (i32.load offset=4 $vb) $np $nl)
                    (set $mutated 1)
                    (br $fc_done))
                  (set $j (i32.add $j 1))
                  (br $fc_lp)))
              (call $buf_push_i32 $vis_cells $mutated)))
          (set $i (i32.add $i 1))
          (br $vl_lp)))

      (set $sb (i32.load offset=28 $fctx))
      (set $scnt (call $buf_count_i32 $sb))
      (set $i $scnt)
      (block $vo_done
        (loop $vo_lp
          (br_if $vo_done (i32.eqz $i))
          (set $i (i32.sub $i 1))
          (set $os (call $buf_get_i32 $sb $i))
          (set $ovb (i32.load $os))
          (set $j (call $buf_count_i32 $ovb))
          (block $voj_done
            (loop $voj_lp
              (br_if $voj_done (i32.eqz $j))
              (set $j (i32.sub $j 1))
              (set $vb (call $buf_get_i32 $ovb $j))
              (when (call $nlist_insert $vis_names (i32.load $vb) (i32.load offset=4 $vb))
                (call $buf_push_i32 $vis_tys (i32.load offset=16 $vb))
                (call $buf_push_i32 $vis_cells (i32.load offset=8 $vb)))
              (br $voj_lp)))
          (br $vo_lp)))

      ;; Reserve WASM functions & CaptureSpecs for each direct nested routine
      (set $i 0)
      (block $rn_done
        (loop $rn_lp
          (br_if $rn_done (i32.ge_u $i $nr_cnt))
          (set $r (call $buf_get_i32 $n_routs $i))
          (set $fn_name (call $make_tag_name 110 (call $buf_count_i32 $out_routines))) ;; "__n_<idx>"
          (set $fn_idx (call $reserve_routine $fn_name $ret_len))
          (set $clst (call $buf_get_i32 $sib_caps $i))
          (set $caps_out (call $buf_new 16))
          (set $j 0)
          (block $cs_done
            (loop $cs_lp
              (br_if $cs_done (i32.ge_u $j (call $buf_count_i32 $clst)))
              (set $vns (call $buf_get_i32 $clst $j))
              (set $k 0)
              (block $vm_done
                (loop $vm_lp
                  (br_if $vm_done (i32.ge_u $k (call $buf_count_i32 $vis_names)))
                  (set $cns (call $buf_get_i32 $vis_names $k))
                  (when (call $slice_eq (i32.load $vns) (i32.load offset=4 $vns) (i32.load $cns) (i32.load offset=4 $cns))
                    (when (call $ty_contains_buf (call $buf_get_i32 $vis_tys $k)) (unreachable))
                    (set $cspec (call $alloc 16))
                    (i32.store $cspec (i32.load $vns))
                    (i32.store offset=4 $cspec (i32.load offset=4 $vns))
                    (i32.store offset=8 $cspec (call $buf_get_i32 $vis_tys $k))
                    (i32.store offset=12 $cspec (call $buf_get_i32 $vis_cells $k))
                    (call $buf_push_i32 $caps_out $cspec)
                    (br $vm_done))
                  (set $k (i32.add $k 1))
                  (br $vm_lp)))
              (set $j (i32.add $j 1))
              (br $cs_lp)))
          (set $nentry (call $alloc 32))
          (i32.store $nentry (i32.load offset=16 $r))
          (i32.store offset=4 $nentry (i32.load offset=20 $r))
          (i32.store offset=8 $nentry $fn_idx)
          (i32.store offset=12 $nentry (i32.load offset=4 $fctx))
          (i32.store offset=16 $nentry $r)
          (i32.store offset=20 $nentry $caps_out)
          (call $buf_push_i32 $srouts $nentry)
          (set $i (i32.add $i 1))
          (br $rn_lp))))

    (set $sb (i32.load offset=28 $fctx))
    (call $buf_push_i32 $sb $slayer)

    ;; 5. Emit lifted nested routine bodies with inherited nested-routine visibility
    (when (i32.gt_u $nr_cnt 0)
      (set $scnt (call $buf_count_i32 $sb))
      (set $inh_scopes (call $buf_new (i32.shl $scnt 2)))
      (set $i 0)
      (block $is_done
        (loop $is_lp
          (br_if $is_done (i32.ge_u $i $scnt))
          (set $os (call $buf_get_i32 $sb $i))
          (set $isl (call $alloc 16))
          (i32.store $isl (call $buf_new 16))
          (i32.store offset=4 $isl (i32.load offset=4 $os))
          (call $buf_push_i32 $inh_scopes $isl)
          (set $i (i32.add $i 1))
          (br $is_lp)))
      (set $i 0)
      (block $en_done
        (loop $en_lp
          (br_if $en_done (i32.ge_u $i $nr_cnt))
          (set $nentry (call $buf_get_i32 $srouts $i))
          (call $emit_routine_body
            $mod_i
            (i32.load offset=8 $nentry)
            (i32.load offset=16 $nentry)
            1
            (i32.load offset=20 $nentry)
            $inh_scopes)
          (set $i (i32.add $i 1))
          (br $en_lp))))

    ;; 6. Emit block statements sequentially
    (set $res_ty $TY_VOID)
    (set $i 0)
    (block $st_done
      (loop $st_lp
        (br_if $st_done (i32.ge_u $i $cnt))
        (set $is_last (i32.eq (i32.add $i 1) $cnt))
        (set $stmt (call $buf_get_i32 $stmts_buf $i))
        (set $cty (call $emit_expr $fctx $stmt (i32.and $in_tail $is_last)))
        (when (i32.eq $cty $TY_NEVER)
          (i32.store offset=4 $sb (i32.sub (i32.load offset=4 $sb) 4))
          (return $TY_NEVER))
        (if $is_last
          (then (set $res_ty $cty))
          (else
            (when (call $ty_wasm_vt $cty)
              (call $fb_op $fb 0x1A))))
        (set $i (i32.add $i 1))
        (br $st_lp)))

    (i32.store offset=4 $sb (i32.sub (i32.load offset=4 $sb) 4))
    (get $res_ty))

  (func $emit_expr (param $fctx i32) (param $ex i32) (param $in_tail i32) (result i32)
    (local $tag       i32)
    (local $mod_i     i32)
    (local $fb        i32)
    (local $alloc_idx i32)
    (local $p         i32)
    (local $len       i32)
    (local $hdr_loc   i32)
    (local $pos       i32)
    (local $elems     i32)
    (local $cnt       i32)
    (local $i         i32)
    (local $cty       i32)
    (local $ety       i32)
    (local $vt        i32)
    (local $locs      i32)
    (local $tys_ptr   i32)
    (local $loc       i32)
    (local $stride    i32)
    (local $aexp      i32)
    (local $sidx      i32)
    (local $sdef      i32)
    (local $t         i32)
    (local $lptr      i32)
    (local $ep        i32)
    (local $base_loc  i32)
    (local $val_loc   i32)
    (local $new_loc   i32)
    (local $v         i32)
    (local $tv        i32)
    (local $sb        i32)
    (local $sl        i32)
    (local $svars     i32)
    (local $floc      i32)
    (local $nr        i32)
    (local $r         i32)
    (local $can_tail  i32)
    (local $caps      i32)
    (local $cap_locs  i32)
    (local $cspec     i32)
    (local $t1        i32)
    (local $t2        i32)
    (local $t3        i32)
    (local $l1        i32)
    (local $l2        i32)
    (local $l3        i32)
    (local $brk_d     i32)
    (local $cnt_d     i32)
    (local $top_d     i32)
    (local $llbl      i32)
    (local $lb        i32)
    (local $op        i32)

    (set $tag (i32.load $ex))
    (set $mod_i (i32.load $fctx))
    (set $fb (i32.load offset=24 $fctx))

    ;; Literals
    (when (i32.eq $tag $EX_BOOL)
      (call $fb_i32_const $fb (i32.load offset=4 $ex))
      (return $TY_B))

    (when (i32.eq $tag $EX_INT)
      (set $cty (i32.load offset=4 $ex))
      (if (i32.eq (call $ty_wasm_vt $cty) $FMT_I64)
        (then (call $fb_i64_const $fb (i64.load offset=24 $ex)))
        (else (call $fb_i32_const $fb (i32.wrap_i64 (i64.load offset=24 $ex)))))
      (return $cty))

    (when (i32.eq $tag $EX_BYTES)
      (set $p (i32.load offset=4 $ex))
      (set $len (i32.load offset=8 $ex))
      (set $alloc_idx (call $ensure_alloc_helper))
      (set $hdr_loc (call $fb_alloc_local $fb $FMT_I32))
      (call $fb_i32_const $fb (i32.add 16 $len))
      (call $fb_op_u32 $fb 0x10 $alloc_idx)
      (call $fb_op_u32 $fb 0x21 $hdr_loc)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb 16)
      (call $fb_op $fb 0x6A)
      (call $fb_mem $fb 0x36 2 0)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb $len)
      (call $fb_mem $fb 0x36 2 4)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb $len)
      (call $fb_mem $fb 0x36 2 8)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb 1)
      (call $fb_mem $fb 0x36 2 12)
      (set $pos 0)
      (block $b8_done
        (loop $b8_lp
          (br_if $b8_done (i32.gt_u (i32.add $pos 8) $len))
          (call $fb_op_u32 $fb 0x20 $hdr_loc)
          (call $fb_i64_const $fb (i64.load align=1 (i32.add $p $pos)))
          (call $fb_mem $fb 0x37 0 (i32.add 16 $pos))
          (set $pos (i32.add $pos 8))
          (br $b8_lp)))
      (block $b1_done
        (loop $b1_lp
          (br_if $b1_done (i32.ge_u $pos $len))
          (call $fb_op_u32 $fb 0x20 $hdr_loc)
          (call $fb_i32_const $fb (i32.load8_u (i32.add $p $pos)))
          (call $fb_mem $fb 0x3A 0 (i32.add 16 $pos))
          (set $pos (i32.add $pos 1))
          (br $b1_lp)))
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (return (call $intern_unary_ty $TK_LIST $TY_U8)))

    (when (i32.eq $tag $EX_LIST)
      (set $cty (call $canon_type $mod_i (i32.load offset=4 $ex)))
      (call $validate_type_wf $cty)
      (when (call $ty_contains_buf $cty) (unreachable))
      (set $vt (call $ty_wasm_vt $cty))
      (when (i32.eqz $vt) (unreachable))
      (set $stride (call $ty_stride $cty))
      (set $aexp (call $ty_align_exp $cty))
      (set $elems (i32.load offset=8 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $locs (call $buf_new (i32.add 16 (i32.shl $cnt 2))))
      (set $i 0)
      (block $le_done
        (loop $le_lp
          (br_if $le_done (i32.ge_u $i $cnt))
          (set $ety (call $emit_expr $fctx (call $buf_get_i32 $elems $i) 0))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (when (i32.ne $ety $cty) (unreachable))
          (set $loc (call $fb_alloc_local $fb $vt))
          (call $fb_op_u32 $fb 0x21 $loc)
          (call $buf_push_i32 $locs $loc)
          (set $i (i32.add $i 1))
          (br $le_lp)))
      (set $alloc_idx (call $ensure_alloc_helper))
      (set $hdr_loc (call $fb_alloc_local $fb $FMT_I32))
      (call $fb_i32_const $fb (i32.add 16 (i32.mul $cnt $stride)))
      (call $fb_op_u32 $fb 0x10 $alloc_idx)
      (call $fb_op_u32 $fb 0x21 $hdr_loc)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb 16)
      (call $fb_op $fb 0x6A)
      (call $fb_mem $fb 0x36 2 0)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb $cnt)
      (call $fb_mem $fb 0x36 2 4)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb $cnt)
      (call $fb_mem $fb 0x36 2 8)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_i32_const $fb 1)
      (call $fb_mem $fb 0x36 2 12)
      (set $i 0)
      (block $ls_done
        (loop $ls_lp
          (br_if $ls_done (i32.ge_u $i $cnt))
          (call $fb_op_u32 $fb 0x20 $hdr_loc)
          (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $locs $i))
          (call $emit_store_slot $fb $stride $aexp (i32.add 16 (i32.mul $i $stride)))
          (set $i (i32.add $i 1))
          (br $ls_lp)))
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (return (call $intern_unary_ty $TK_LIST $cty)))

    (when (i32.eq $tag $EX_TUPLE_NEW)
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (when (i32.eqz $cnt)
        (call $fb_i32_const $fb 0)
        (return $TY_TUP0))
      (set $locs (call $buf_new (i32.shl $cnt 2)))
      (set $tys_ptr (call $alloc (i32.shl $cnt 2)))
      (set $i 0)
      (block $tn_done
        (loop $tn_lp
          (br_if $tn_done (i32.ge_u $i $cnt))
          (set $ety (call $emit_expr $fctx (call $buf_get_i32 $elems $i) 0))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (set $vt (call $ty_wasm_vt $ety))
          (when (i32.eqz $vt) (unreachable))
          (set $loc (call $fb_alloc_local $fb $vt))
          (call $fb_op_u32 $fb 0x21 $loc)
          (call $buf_push_i32 $locs $loc)
          (i32.store (i32.add $tys_ptr (i32.shl $i 2)) $ety)
          (set $i (i32.add $i 1))
          (br $tn_lp)))
      (set $cty (call $intern_tuple_ty $cnt $tys_ptr))
      (call $emit_store_composite_slots $fb $cty $locs)
      (return $cty))

    (when (i32.eq $tag $EX_STRUCT_NEW)
      (set $sidx
        (if (result i32) (i32.load offset=4 $ex)
          (then (call $resolve_struct_dc $mod_i (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
          (else (call $resolve_struct_ident $mod_i (i32.load offset=8 $ex) (i32.load offset=12 $ex)))))
      (when (i32.lt_s $sidx 0) (unreachable))
      (set $sdef (call $buf_get_i32 $all_structs $sidx))
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (when (i32.ne $cnt (i32.load offset=32 $sdef)) (unreachable))
      (set $tys_ptr (i32.load offset=28 $sdef))
      (set $locs (call $buf_new (i32.shl $cnt 2)))
      (set $i 0)
      (block $sn_done
        (loop $sn_lp
          (br_if $sn_done (i32.ge_u $i $cnt))
          (set $ety (call $emit_expr $fctx (call $buf_get_i32 $elems $i) 0))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (when (i32.ne $ety (i32.load (i32.add $tys_ptr (i32.shl $i 2))))
            (unreachable))
          (set $loc (call $fb_alloc_local $fb (call $ty_wasm_vt $ety)))
          (call $fb_op_u32 $fb 0x21 $loc)
          (call $buf_push_i32 $locs $loc)
          (set $i (i32.add $i 1))
          (br $sn_lp)))
      (set $cty (call $intern_struct_ty $sidx))
      (call $emit_store_composite_slots $fb $cty $locs)
      (return $cty))

    (when (i32.eq $tag $EX_FIELD)
      (set $cty (call $emit_expr $fctx (i32.load offset=4 $ex) 0))
      (when (i32.eq $cty $TY_NEVER) (return $TY_NEVER))
      (call $ensure_composite_layout $cty)
      (set $t (call $buf_get_i32 $type_table $cty))
      (set $cnt
        (if (result i32) (i32.eq (i32.load $t) $TK_TUPLE)
          (then (i32.load offset=4 $t))
          (else (i32.load offset=32 (call $buf_get_i32 $all_structs (i32.load offset=4 $t))))))
      (set $i (i32.load offset=8 $ex))
      (when (i32.ge_u $i $cnt) (unreachable))
      (set $ep (i32.add (i32.load offset=20 $t) (i32.shl $i 4)))
      (call $emit_load_slot $fb (i32.load offset=4 $ep) (i32.load offset=8 $ep) (i32.load $ep))
      (return (i32.load offset=12 $ep)))

    (when (i32.eq $tag $EX_WITH)
      (set $cty (call $emit_expr $fctx (i32.load offset=4 $ex) 0))
      (when (i32.eq $cty $TY_NEVER) (return $TY_NEVER))
      (call $ensure_composite_layout $cty)
      (set $t (call $buf_get_i32 $type_table $cty))
      (set $cnt
        (if (result i32) (i32.eq (i32.load $t) $TK_TUPLE)
          (then (i32.load offset=4 $t))
          (else (i32.load offset=32 (call $buf_get_i32 $all_structs (i32.load offset=4 $t))))))
      (set $i (i32.load offset=8 $ex))
      (when (i32.ge_u $i $cnt) (unreachable))
      (set $ep (i32.add (i32.load offset=20 $t) (i32.shl $i 4)))
      (set $base_loc (call $fb_alloc_local $fb $FMT_I32))
      (call $fb_op_u32 $fb 0x21 $base_loc)
      (set $ety (call $emit_expr $fctx (i32.load offset=12 $ex) 0))
      (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
      (when (i32.ne $ety (i32.load offset=12 $ep)) (unreachable))
      (set $val_loc (call $fb_alloc_local $fb (call $ty_wasm_vt $ety)))
      (call $fb_op_u32 $fb 0x21 $val_loc)
      (set $len (i32.load offset=16 $t))
      (set $alloc_idx (call $ensure_alloc_helper))
      (set $new_loc (call $fb_alloc_local $fb $FMT_I32))
      (call $fb_i32_const $fb $len)
      (call $fb_op_u32 $fb 0x10 $alloc_idx)
      (call $fb_op_u32 $fb 0x21 $new_loc)
      (call $fb_op_u32 $fb 0x20 $new_loc)
      (call $fb_op_u32 $fb 0x20 $base_loc)
      (call $fb_i32_const $fb $len)
      (call $fb_mem_copy $fb)
      (call $fb_op_u32 $fb 0x20 $new_loc)
      (call $fb_op_u32 $fb 0x20 $val_loc)
      (call $emit_store_slot $fb (i32.load offset=4 $ep) (i32.load offset=8 $ep) (i32.load $ep))
      (call $fb_op_u32 $fb 0x20 $new_loc)
      (return $cty))

    (when (i32.eq $tag $EX_BUF_NEW)
      (when (i32.load offset=40 $fctx) (unreachable))
      (set $ety (call $emit_expr $fctx (i32.load offset=8 $ex) 0))
      (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
      (when (i32.ne $ety $TY_U64) (unreachable))
      (set $cty (call $canon_type $mod_i (i32.load offset=4 $ex)))
      (call $validate_type_wf $cty)
      (when (i32.eqz (call $ty_wasm_vt $cty)) (unreachable))
      (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_STAGEL_BUF_NEW $DC_STAGEL_BUF_TYPE $cty))
      (return (call $intern_unary_ty $TK_BUF $cty)))

    (when (i32.eq $tag $EX_VALUE_REF)
      (set $op 0)
      (when (i32.load offset=4 $ex)
        (set $op (i32.add 1 (call $canon_type $mod_i (i32.load offset=4 $ex)))))
      (when (i32.eqz (i32.load offset=8 $ex))
        (set $v (call $lookup_var $fctx (i32.load offset=12 $ex) (i32.load offset=16 $ex)))
        (when $v
          (when (i32.and $op (i32.ne (i32.sub $op 1) (i32.load offset=16 $v)))
            (unreachable))
          (return (call $emit_read_var $fctx $v))))
      (set $tv
        (call $resolve_top_val
          $mod_i $op
          (i32.load offset=8 $ex) (i32.load offset=12 $ex) (i32.load offset=16 $ex)))
      (call $fb_op_u32 $fb 0x23 (i32.load offset=32 $tv))
      (return (i32.load offset=28 $tv)))

    (when (i32.eq $tag $EX_LET)
      (when (i32.load offset=40 $fctx) (unreachable))
      (set $cty (call $canon_type $mod_i (i32.load offset=4 $ex)))
      (call $validate_type_wf $cty)
      (set $vt (call $ty_wasm_vt $cty))
      (when (i32.eqz $vt) (unreachable))
      ;; Check if pre-allocated as a shadow cell in innermost scope
      (set $sb (i32.load offset=28 $fctx))
      (set $cnt (call $buf_count_i32 $sb))
      (when (i32.gt_u $cnt 0)
        (set $sl (call $buf_get_i32 $sb (i32.sub $cnt 1)))
        (set $svars (i32.load $sl))
        (set $i 0)
        (block $pc_done
          (loop $pc_lp
            (br_if $pc_done (i32.ge_u $i (call $buf_count_i32 $svars)))
            (set $v (call $buf_get_i32 $svars $i))
            (when
              (i32.and
                (i32.load offset=8 $v)
                (call $slice_eq (i32.load $v) (i32.load offset=4 $v) (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
              (when (i32.load offset=16 $ex)
                (set $ety (call $emit_expr $fctx (i32.load offset=16 $ex) 0))
                (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
                (when (i32.ne $ety $cty) (unreachable))
                (set $loc (call $fb_alloc_local $fb $vt))
                (call $fb_op_u32 $fb 0x21 $loc)
                (call $emit_write_cell $fb (i32.load offset=12 $v) $cty $loc))
              (return $TY_VOID))
            (set $i (i32.add $i 1))
            (br $pc_lp))))
      (set $loc (call $fb_alloc_local $fb $vt))
      (set $floc -1)
      (if (i32.load offset=16 $ex)
        (then
          (set $ety (call $emit_expr $fctx (i32.load offset=16 $ex) 0))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (when (i32.ne $ety $cty) (unreachable))
          (call $fb_op_u32 $fb 0x21 $loc))
        (else
          (set $floc (call $fb_alloc_local $fb $FMT_I32))))
      (when (i32.gt_u $cnt 0)
        (set $sl (call $buf_get_i32 $sb (i32.sub $cnt 1)))
        (set $v (call $alloc 32))
        (i32.store $v (i32.load offset=8 $ex))
        (i32.store offset=4 $v (i32.load offset=12 $ex))
        (i32.store offset=8 $v 0)
        (i32.store offset=12 $v $loc)
        (i32.store offset=16 $v $cty)
        (i32.store offset=20 $v 1)
        (i32.store offset=24 $v $floc)
        (call $buf_push_i32 (i32.load $sl) $v))
      (return $TY_VOID))

    (when (i32.eq $tag $EX_CONST)
      (set $cty (call $canon_type $mod_i (i32.load offset=4 $ex)))
      (call $validate_type_wf $cty)
      (set $vt (call $ty_wasm_vt $cty))
      (when (i32.eqz $vt) (unreachable))
      (set $ety (call $emit_expr $fctx (i32.load offset=16 $ex) 0))
      (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
      (when (i32.ne $ety $cty) (unreachable))
      (set $loc (call $fb_alloc_local $fb $vt))
      (call $fb_op_u32 $fb 0x21 $loc)
      (set $sb (i32.load offset=28 $fctx))
      (set $cnt (call $buf_count_i32 $sb))
      (when (i32.gt_u $cnt 0)
        (set $sl (call $buf_get_i32 $sb (i32.sub $cnt 1)))
        (set $v (call $alloc 32))
        (i32.store $v (i32.load offset=8 $ex))
        (i32.store offset=4 $v (i32.load offset=12 $ex))
        (i32.store offset=8 $v 0)
        (i32.store offset=12 $v $loc)
        (i32.store offset=16 $v $cty)
        (i32.store offset=20 $v 0)
        (i32.store offset=24 $v -1)
        (call $buf_push_i32 (i32.load $sl) $v))
      (return $TY_VOID))

    (when (i32.eq $tag $EX_SET)
      (when (i32.load offset=40 $fctx) (unreachable))
      (set $v (call $lookup_var $fctx (i32.load offset=8 $ex) (i32.load offset=12 $ex)))
      (when (i32.eqz $v) (unreachable))
      (when (i32.eqz (i32.load offset=20 $v)) (unreachable))
      (set $cty (i32.load offset=16 $v))
      (when (i32.load offset=4 $ex)
        (when (i32.ne (call $canon_type $mod_i (i32.load offset=4 $ex)) $cty)
          (unreachable)))
      (set $ety (call $emit_expr $fctx (i32.load offset=16 $ex) 0))
      (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
      (when (i32.ne $ety $cty) (unreachable))
      (if (i32.load offset=8 $v)
        (then
          (set $loc (call $fb_alloc_local $fb (call $ty_wasm_vt $cty)))
          (call $fb_op_u32 $fb 0x21 $loc)
          (call $emit_write_cell $fb (i32.load offset=12 $v) $cty $loc))
        (else
          (call $fb_op_u32 $fb 0x21 (i32.load offset=12 $v))
          (set $floc (i32.load offset=24 $v))
          (when (i32.ge_s $floc 0)
            (call $fb_i32_const $fb 1)
            (call $fb_op_u32 $fb 0x21 $floc))))
      (return $TY_VOID))

    (when (i32.or (i32.eq $tag $EX_ROUTINE) (i32.eq $tag $EX_NOP))
      (when (i32.and (i32.eq $tag $EX_ROUTINE) (i32.load offset=40 $fctx))
        (unreachable))
      (return $TY_VOID))

    (when (i32.eq $tag $EX_BLOCK)
      (return (call $emit_block_expr $fctx (i32.load offset=4 $ex) $in_tail)))

    (when (i32.eq $tag $EX_CALL)
      (when (i32.load offset=40 $fctx) (unreachable))
      (set $elems (i32.load offset=16 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $locs (call $buf_new (i32.add 16 (i32.shl $cnt 2))))
      (set $tys_ptr (call $alloc (i32.add 4 (i32.shl $cnt 2))))
      ;; Infer static arg types first for overload resolution (in case an arg diverges)
      (set $i 0)
      (block $ci_done
        (loop $ci_lp
          (br_if $ci_done (i32.ge_u $i $cnt))
          (i32.store
            (i32.add $tys_ptr (i32.shl $i 2))
            (call $infer_expr_type $fctx (call $buf_get_i32 $elems $i)))
          (set $i (i32.add $i 1))
          (br $ci_lp)))
      (set $nr 0)
      (when (i32.eqz (i32.load offset=4 $ex))
        (set $nr (call $lookup_nested_rout $fctx (i32.load offset=8 $ex) (i32.load offset=12 $ex))))
      (if $nr
        (then
          (set $r (i32.load offset=16 $nr))
          (when (i32.eqz (call $routine_matches_args $r $cnt $tys_ptr)) (unreachable)))
        (else
          (set $r
            (call $resolve_top_rout
              $mod_i
              (i32.load offset=4 $ex) (i32.load offset=8 $ex) (i32.load offset=12 $ex)
              $cnt $tys_ptr))))
      (set $cty (i32.load offset=32 $r))
      (set $can_tail
        (i32.and
          $in_tail
          (i32.eq (call $ty_wasm_vt (i32.load offset=12 $fctx)) (call $ty_wasm_vt $cty))))

      (when (i32.and (i32.eqz $nr) (i32.or (i32.eqz $can_tail) (i32.eqz $uses_shadow)))
        ;; Top-level call needing no capture preparation or shadow pop: push args directly
        (set $i 0)
        (block $dc_done
          (loop $dc_lp
            (br_if $dc_done (i32.ge_u $i $cnt))
            (set $ety (call $emit_expr $fctx (call $buf_get_i32 $elems $i) 0))
            (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
            (set $i (i32.add $i 1))
            (br $dc_lp)))
        (if $can_tail
          (then
            (call $fb_op_u32 $fb 0x12 (i32.load offset=36 $r))
            (return $TY_NEVER))
          (else
            (call $fb_op_u32 $fb 0x10 (i32.load offset=36 $r))
            (when (i32.eq $cty $TY_NEVER) (call $fb_op $fb 0x00))
            (return $cty))))

      ;; Evaluate arguments left-to-right into temp locals
      (set $i 0)
      (block $ce_done
        (loop $ce_lp
          (br_if $ce_done (i32.ge_u $i $cnt))
          (set $ety (call $emit_expr $fctx (call $buf_get_i32 $elems $i) 0))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (set $loc (call $fb_alloc_local $fb (call $ty_wasm_vt $ety)))
          (call $fb_op_u32 $fb 0x21 $loc)
          (call $buf_push_i32 $locs $loc)
          (set $i (i32.add $i 1))
          (br $ce_lp)))

      (if (i32.eqz $nr)
        (then
          (if $can_tail
            (then
              (call $emit_routine_shadow_pop $fctx)
              (set $i 0)
              (block $ta_done
                (loop $ta_lp
                  (br_if $ta_done (i32.ge_u $i $cnt))
                  (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $locs $i))
                  (set $i (i32.add $i 1))
                  (br $ta_lp)))
              (call $fb_op_u32 $fb 0x12 (i32.load offset=36 $r))
              (return $TY_NEVER))
            (else
              (set $i 0)
              (block $na_done
                (loop $na_lp
                  (br_if $na_done (i32.ge_u $i $cnt))
                  (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $locs $i))
                  (set $i (i32.add $i 1))
                  (br $na_lp)))
              (call $fb_op_u32 $fb 0x10 (i32.load offset=36 $r))
              (when (i32.eq $cty $TY_NEVER) (call $fb_op $fb 0x00))
              (return $cty))))
        (else
          ;; Nested routine call: prepare captured arguments
          (set $caps (i32.load offset=20 $nr))
          (set $cap_locs (call $buf_new 16))
          (set $i 0)
          (block $cp_done
            (loop $cp_lp
              (br_if $cp_done (i32.ge_u $i (call $buf_count_i32 $caps)))
              (set $cspec (call $buf_get_i32 $caps $i))
              (set $v (call $lookup_var $fctx (i32.load $cspec) (i32.load offset=4 $cspec)))
              (when (i32.eqz $v) (unreachable))
              (if (i32.load offset=12 $cspec)
                (then
                  (when (i32.eqz (i32.load offset=8 $v)) (unreachable))
                  (call $buf_push_i32 $cap_locs (i32.load offset=12 $v)))
                (else
                  (drop (call $emit_read_var $fctx $v))
                  (set $loc (call $fb_alloc_local $fb (call $ty_wasm_vt (i32.load offset=8 $cspec))))
                  (call $fb_op_u32 $fb 0x21 $loc)
                  (call $buf_push_i32 $cap_locs $loc)))
              (set $i (i32.add $i 1))
              (br $cp_lp)))

          (if $can_tail
            (then
              (when $uses_shadow
                (if (i32.eq (i32.load offset=12 $nr) (i32.load offset=4 $fctx))
                  (then
                    ;; Direct child nested routine: keep current shadow cells live
                    (set $loc
                      (if (result i32) (i32.load offset=8 $fctx)
                        (then (i32.load offset=20 $fctx))
                        (else (i32.load offset=16 $fctx))))
                    (call $fb_op_u32 $fb 0x20 $loc))
                  (else
                    ;; Self/sibling/ancestor nested routine: pop current routine's cells
                    (set $loc (i32.load offset=16 $fctx))
                    (when (i32.ge_s $loc 0)
                      (call $fb_op_u32 $fb 0x20 $loc)
                      (call $fb_op_u32 $fb 0x24 $sh_sp_glob))
                    (set $loc
                      (if (result i32) (i32.ge_s (i32.load offset=20 $fctx) 0)
                        (then (i32.load offset=20 $fctx))
                        (else (i32.load offset=16 $fctx))))
                    (call $fb_op_u32 $fb 0x20 $loc))))
              (set $i 0)
              (block $tc_done
                (loop $tc_lp
                  (br_if $tc_done (i32.ge_u $i (call $buf_count_i32 $cap_locs)))
                  (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $cap_locs $i))
                  (set $i (i32.add $i 1))
                  (br $tc_lp)))
              (set $i 0)
              (block $ta2_done
                (loop $ta2_lp
                  (br_if $ta2_done (i32.ge_u $i $cnt))
                  (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $locs $i))
                  (set $i (i32.add $i 1))
                  (br $ta2_lp)))
              (call $fb_op_u32 $fb 0x12 (i32.load offset=8 $nr))
              (return $TY_NEVER))
            (else
              (when $uses_shadow
                (call $fb_op_u32 $fb 0x23 $sh_sp_glob))
              (set $i 0)
              (block $nc_done
                (loop $nc_lp
                  (br_if $nc_done (i32.ge_u $i (call $buf_count_i32 $cap_locs)))
                  (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $cap_locs $i))
                  (set $i (i32.add $i 1))
                  (br $nc_lp)))
              (set $i 0)
              (block $na2_done
                (loop $na2_lp
                  (br_if $na2_done (i32.ge_u $i $cnt))
                  (call $fb_op_u32 $fb 0x20 (call $buf_get_i32 $locs $i))
                  (set $i (i32.add $i 1))
                  (br $na2_lp)))
              (call $fb_op_u32 $fb 0x10 (i32.load offset=8 $nr))
              (when (i32.eq $cty $TY_NEVER) (call $fb_op $fb 0x00))
              (return $cty))))))

    (when (i32.eq $tag $EX_IF)
      (set $ety (call $emit_expr $fctx (i32.load offset=4 $ex) 0))
      (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
      (when (i32.ne $ety $TY_B) (unreachable))
      (set $t1 (call $infer_expr_type $fctx (i32.load offset=8 $ex)))
      (set $t2 (call $infer_expr_type $fctx (i32.load offset=12 $ex)))
      (set $cty
        (if (result i32) (i32.eq $t1 $TY_NEVER)
          (then (get $t2))
          (else
            (if (result i32) (i32.eq $t2 $TY_NEVER)
              (then (get $t1))
              (else
                (if (result i32) (i32.eq $t1 $t2) (then (get $t1)) (else (unreachable))))))))
      (call $fb_block $fb 0x04 (call $ty_wasm_vt $cty))
      (i32.store offset=36 $fctx (i32.add (i32.load offset=36 $fctx) 1))
      (set $t1 (call $emit_expr $fctx (i32.load offset=8 $ex) $in_tail))
      (call $fb_op $fb 0x05)
      (set $t2 (call $emit_expr $fctx (i32.load offset=12 $ex) $in_tail))
      (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
      (call $fb_op $fb 0x0B)
      (when (i32.and (i32.eq $t1 $TY_NEVER) (i32.eq $t2 $TY_NEVER))
        (call $fb_op $fb 0x00)
        (return $TY_NEVER))
      (return $cty))

    (when (i32.eq $tag $EX_WHILE)
      (when (i32.load offset=40 $fctx) (unreachable))
      (call $fb_block $fb 0x02 0)
      (set $brk_d (i32.add (i32.load offset=36 $fctx) 1))
      (i32.store offset=36 $fctx $brk_d)
      (call $fb_block $fb 0x03 0)
      (set $cnt_d (i32.add $brk_d 1))
      (i32.store offset=36 $fctx $cnt_d)
      (set $ety (call $emit_expr $fctx (i32.load offset=4 $ex) 0))
      (when (i32.ne $ety $TY_NEVER)
        (when (i32.ne $ety $TY_B) (unreachable))
        (call $fb_op $fb 0x45)
        (call $fb_op_u32 $fb 0x0D (i32.sub (i32.load offset=36 $fctx) $brk_d))
        (set $llbl (call $alloc 8))
        (i32.store $llbl $brk_d)
        (i32.store offset=4 $llbl $cnt_d)
        (set $lb (i32.load offset=32 $fctx))
        (call $buf_push_i32 $lb $llbl)
        (set $t1 (call $emit_expr $fctx (i32.load offset=8 $ex) 0))
        (i32.store offset=4 $lb (i32.sub (i32.load offset=4 $lb) 4))
        (when (i32.ne $t1 $TY_NEVER)
          (when (i32.ne $t1 $TY_VOID) (unreachable))
          (call $fb_op_u32 $fb 0x0C (i32.sub (i32.load offset=36 $fctx) $cnt_d))))
      (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
      (call $fb_op $fb 0x0B)
      (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
      (call $fb_op $fb 0x0B)
      (return $TY_VOID))

    (when (i32.eq $tag $EX_FOREACH)
      (when (i32.load offset=40 $fctx) (unreachable))
      (set $t1 (call $emit_expr $fctx (i32.load offset=16 $ex) 0))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (set $cty (call $canon_type $mod_i (i32.load offset=4 $ex)))
      (call $validate_type_wf $cty)
      (when (i32.ne $t1 (call $intern_unary_ty $TK_LIST $cty)) (unreachable))
      (set $vt (call $ty_wasm_vt $cty))
      (set $stride (call $ty_stride $cty))
      (set $aexp (call $ty_align_exp $cty))

      (set $hdr_loc (call $fb_alloc_local $fb $FMT_I32))
      (set $l1 (call $fb_alloc_local $fb $FMT_I32)) ;; data_ptr
      (set $l2 (call $fb_alloc_local $fb $FMT_I32)) ;; len
      (set $l3 (call $fb_alloc_local $fb $FMT_I32)) ;; idx
      (set $loc (call $fb_alloc_local $fb $vt))     ;; elem

      (call $fb_op_u32 $fb 0x21 $hdr_loc)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_mem $fb 0x28 2 0)
      (call $fb_op_u32 $fb 0x21 $l1)
      (call $fb_op_u32 $fb 0x20 $hdr_loc)
      (call $fb_mem $fb 0x28 2 4)
      (call $fb_op_u32 $fb 0x21 $l2)
      (call $fb_i32_const $fb 0)
      (call $fb_op_u32 $fb 0x21 $l3)

      (call $fb_block $fb 0x02 0)
      (set $brk_d (i32.add (i32.load offset=36 $fctx) 1))
      (i32.store offset=36 $fctx $brk_d)
      (call $fb_block $fb 0x03 0)
      (set $top_d (i32.add $brk_d 1))
      (i32.store offset=36 $fctx $top_d)

      (call $fb_op_u32 $fb 0x20 $l3)
      (call $fb_op_u32 $fb 0x20 $l2)
      (call $fb_op $fb 0x4F)
      (call $fb_op_u32 $fb 0x0D (i32.sub (i32.load offset=36 $fctx) $brk_d))

      (call $fb_op_u32 $fb 0x20 $l1)
      (call $fb_op_u32 $fb 0x20 $l3)
      (call $fb_i32_const $fb $stride)
      (call $fb_op $fb 0x6C)
      (call $fb_op $fb 0x6A)
      (call $emit_load_slot $fb $stride $aexp 0)
      (call $fb_op_u32 $fb 0x21 $loc)

      (call $fb_block $fb 0x02 0)
      (set $cnt_d (i32.add (i32.load offset=36 $fctx) 1))
      (i32.store offset=36 $fctx $cnt_d)

      (set $sl (call $alloc 16))
      (set $svars (call $buf_new 8))
      (set $v (call $alloc 32))
      (i32.store $v (i32.load offset=8 $ex))
      (i32.store offset=4 $v (i32.load offset=12 $ex))
      (i32.store offset=12 $v $loc)
      (i32.store offset=16 $v $cty)
      (i32.store offset=24 $v -1)
      (call $buf_push_i32 $svars $v)
      (i32.store $sl $svars)
      (i32.store offset=4 $sl (call $buf_new 8))
      (set $sb (i32.load offset=28 $fctx))
      (call $buf_push_i32 $sb $sl)

      (set $llbl (call $alloc 8))
      (i32.store $llbl $brk_d)
      (i32.store offset=4 $llbl $cnt_d)
      (set $lb (i32.load offset=32 $fctx))
      (call $buf_push_i32 $lb $llbl)

      (set $ety (call $emit_expr $fctx (i32.load offset=20 $ex) 0))
      (when (i32.and (i32.ne $ety $TY_NEVER) (i32.ne $ety $TY_VOID))
        (unreachable))

      (i32.store offset=4 $lb (i32.sub (i32.load offset=4 $lb) 4))
      (i32.store offset=4 $sb (i32.sub (i32.load offset=4 $sb) 4))

      (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
      (call $fb_op $fb 0x0B)

      (call $fb_op_u32 $fb 0x20 $l3)
      (call $fb_i32_const $fb 1)
      (call $fb_op $fb 0x6A)
      (call $fb_op_u32 $fb 0x21 $l3)
      (call $fb_op_u32 $fb 0x0C (i32.sub (i32.load offset=36 $fctx) $top_d))

      (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
      (call $fb_op $fb 0x0B)
      (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
      (call $fb_op $fb 0x0B)
      (return $TY_VOID))

    (when (i32.eq $tag $EX_BREAK)
      (set $lb (i32.load offset=32 $fctx))
      (set $cnt (call $buf_count_i32 $lb))
      (when (i32.eqz $cnt) (unreachable))
      (set $llbl (call $buf_get_i32 $lb (i32.sub $cnt 1)))
      (call $fb_op_u32 $fb 0x0C (i32.sub (i32.load offset=36 $fctx) (i32.load $llbl)))
      (return $TY_NEVER))

    (when (i32.eq $tag $EX_CONTINUE)
      (set $lb (i32.load offset=32 $fctx))
      (set $cnt (call $buf_count_i32 $lb))
      (when (i32.eqz $cnt) (unreachable))
      (set $llbl (call $buf_get_i32 $lb (i32.sub $cnt 1)))
      (call $fb_op_u32 $fb 0x0C (i32.sub (i32.load offset=36 $fctx) (i32.load offset=4 $llbl)))
      (return $TY_NEVER))

    (when (i32.eq $tag $EX_RETURN)
      (when (i32.load offset=40 $fctx) (unreachable))
      (set $cty (i32.load offset=12 $fctx))
      (if (i32.load offset=4 $ex)
        (then
          (when (i32.eq $cty $TY_VOID) (unreachable))
          (set $ety (call $emit_expr $fctx (i32.load offset=4 $ex) 1))
          (when (i32.eq $ety $TY_NEVER) (return $TY_NEVER))
          (when (i32.ne $ety $cty) (unreachable))
          (set $vt (call $ty_wasm_vt $cty))
          (if $vt
            (then
              (set $loc (call $fb_alloc_local $fb $vt))
              (call $fb_op_u32 $fb 0x21 $loc)
              (call $emit_routine_shadow_pop $fctx)
              (call $fb_op_u32 $fb 0x20 $loc))
            (else
              (call $emit_routine_shadow_pop $fctx))))
        (else
          (when (i32.ne $cty $TY_VOID) (unreachable))
          (call $emit_routine_shadow_pop $fctx)))
      (call $fb_op $fb 0x0F)
      (return $TY_NEVER))

    (when (i32.eq $tag $EX_FAULT)
      (call $fb_op $fb 0x00)
      (return $TY_NEVER))

    (when (i32.eq $tag $EX_UNARY)
      (set $op (i32.load offset=4 $ex))
      (set $t1 (call $emit_expr $fctx (i32.load offset=8 $ex) 0))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (when (i32.eq $op $DC_SEQ_LEN_OP)
        (when (i32.lt_u $t1 8) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (set $cnt (i32.load $t))
        (set $p
          (if (result i32) (i32.eq $cnt $TK_LIST)
            (then (i32.const $DC_STAGEL_LIST_TYPE))
            (else
              (if (result i32) (i32.eq $cnt $TK_BUF)
                (then (i32.const $DC_STAGEL_BUF_TYPE))
                (else (unreachable))))))
        (call $emit_builtin_op $fb 1 $DC_SEQ_LEN_OP 2 $p $DC_TYPE_ANY 0)
        (return $TY_U64))
      (when (i32.eq $op $DC_STAGEL_BUF_FREEZE)
        (when (i32.load offset=40 $fctx) (unreachable))
        (when (i32.lt_u $t1 8) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when (i32.ne (i32.load $t) $TK_BUF) (unreachable))
        (set $ety (i32.load offset=4 $t))
        (when (call $ty_contains_buf $ety) (unreachable))
        (call $fb_op_u32 $fb 0x10 (call $link_builtin $DC_STAGEL_BUF_FREEZE 2 $DC_STAGEL_BUF_TYPE $DC_TYPE_ANY 0))
        (return (call $intern_unary_ty $TK_LIST $ety)))
      (when (i32.eq $op $DC_REQUEST_HOST_CAPABILITY_OP)
        (when (i32.lt_u $t1 8) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when
          (i32.or
            (i32.ne (i32.load $t) $TK_LIST)
            (i32.ne (i32.load offset=4 $t) $TY_U8))
          (unreachable))
        (call $fb_op_u32 $fb 0x10 (call $link_builtin $DC_REQUEST_HOST_CAPABILITY_OP 2 $DC_STAGEL_LIST_TYPE $FMT_U8 0))
        (return $t1))
      (call $emit_scalar_op $fb 1 $op $t1)
      (when (i32.eq $op $DC_LOGICAL_NOT_OP) (return $TY_B))
      (when (i32.and (i32.ge_u $op $DC_CLZ_OP) (i32.le_u $op $DC_POPCNT_OP)) (return $TY_U32))
      (return $t1))

    (when (i32.eq $tag $EX_BINARY)
      (set $op (i32.load offset=4 $ex))
      (set $t1 (call $emit_expr $fctx (i32.load offset=8 $ex) 0))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))

      ;; Short-circuit `and` / `or`
      (when (i32.or (i32.eq $op $DC_LOGICAL_AND_OP) (i32.eq $op $DC_LOGICAL_OR_OP))
        (when (i32.ne $t1 $TY_B) (unreachable))
        (call $fb_block $fb 0x04 $FMT_I32)
        (i32.store offset=36 $fctx (i32.add (i32.load offset=36 $fctx) 1))
        (if (i32.eq $op $DC_LOGICAL_AND_OP)
          (then
            (set $t2 (call $emit_expr $fctx (i32.load offset=12 $ex) 0))
            (when (i32.and (i32.ne $t2 $TY_NEVER) (i32.ne $t2 $TY_B)) (unreachable))
            (call $fb_op $fb 0x05)
            (call $fb_i32_const $fb 0))
          (else
            (call $fb_i32_const $fb 1)
            (call $fb_op $fb 0x05)
            (set $t2 (call $emit_expr $fctx (i32.load offset=12 $ex) 0))
            (when (i32.and (i32.ne $t2 $TY_NEVER) (i32.ne $t2 $TY_B)) (unreachable))))
        (i32.store offset=36 $fctx (i32.sub (i32.load offset=36 $fctx) 1))
        (call $fb_op $fb 0x0B)
        (return $TY_B))

      (when (i32.eqz (call $ty_wasm_vt $t1)) (unreachable))
      (set $t2 (call $emit_expr $fctx (i32.load offset=12 $ex) 0))
      (when (i32.eq $t2 $TY_NEVER) (return $TY_NEVER))
      (when (i32.eqz (call $ty_wasm_vt $t2)) (unreachable))

      (when (i32.or (i32.eq $op $DC_EQ_OP) (i32.eq $op $DC_NE_OP))
        (when (i32.ne $t1 $t2) (unreachable))
        (when (call $ty_contains_buf $t1) (unreachable))
        (when (call $ty_is_scalar $t1)
          (call $emit_scalar_op $fb 2 $op $t1)
          (return $TY_B))
        (when (i32.eq $t1 $TY_TUP0)
          (call $fb_op $fb 0x1A)
          (call $fb_op $fb 0x1A)
          (call $fb_i32_const $fb (i32.eq $op $DC_EQ_OP))
          (return $TY_B))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when (i32.eq (i32.load $t) $TK_LIST)
          (set $ety (i32.load offset=4 $t))
          (when (call $ty_is_scalar $ety)
            (call $fb_op_u32
              $fb 0x10
              (call $link_builtin $op 2 $DC_STAGEL_LIST_TYPE (call $ty_scalar_fmt_cp $ety) 0))
            (return $TY_B)))
        (call $fb_op_u32 $fb 0x10 (call $ensure_structural_eq_helper $t1))
        (when (i32.eq $op $DC_NE_OP) (call $fb_op $fb 0x45))
        (return $TY_B))

      (when (i32.eq $op $DC_SEQ_GET_OP)
        (when (i32.or (i32.lt_u $t1 8) (i32.ne $t2 $TY_U64)) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (set $cnt (i32.load $t))
        (set $ety (i32.load offset=4 $t))
        (if (i32.eq $cnt $TK_LIST)
          (then
            (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_SEQ_GET_OP $DC_STAGEL_LIST_TYPE $ety))
            (return $ety))
          (else
            (when (i32.ne $cnt $TK_BUF) (unreachable))
            (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_SEQ_GET_OP $DC_STAGEL_BUF_TYPE $ety))
            (return $ety))))

      (when (i32.eq $op $DC_SEQ_CONCAT_OP)
        (when (i32.or (i32.lt_u $t1 8) (i32.ne $t1 $t2)) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when (i32.ne (i32.load $t) $TK_LIST) (unreachable))
        (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_SEQ_CONCAT_OP $DC_STAGEL_LIST_TYPE (i32.load offset=4 $t)))
        (return $t1))

      (when (i32.eq $op $DC_STAGEL_BUF_FILL)
        (when (i32.or (i32.load offset=40 $fctx) (i32.ne $t2 $TY_U64)) (unreachable))
        (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_STAGEL_BUF_FILL $DC_STAGEL_BUF_TYPE $t1))
        (return (call $intern_unary_ty $TK_BUF $t1)))

      (when (i32.eq $op $DC_STAGEL_BUF_PUSH)
        (when (i32.or (i32.load offset=40 $fctx) (i32.lt_u $t1 8)) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when (i32.or (i32.ne (i32.load $t) $TK_BUF) (i32.ne (i32.load offset=4 $t) $t2))
          (unreachable))
        (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_STAGEL_BUF_PUSH $DC_STAGEL_BUF_TYPE $t2))
        (return $TY_VOID))

      (when (i32.eq $op $DC_STAGEL_BUF_EXTEND)
        (when (i32.or (i32.load offset=40 $fctx) (i32.lt_u $t1 8)) (unreachable))
        (set $t (call $buf_get_i32 $type_table $t1))
        (when (i32.ne (i32.load $t) $TK_BUF) (unreachable))
        (set $ety (i32.load offset=4 $t))
        (when (i32.ne $t2 (call $intern_unary_ty $TK_LIST $ety)) (unreachable))
        (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_STAGEL_BUF_EXTEND $DC_STAGEL_BUF_TYPE $ety))
        (return $TY_VOID))

      ;; Remaining binary scalar ops (`add..rem`, `wrap*`, `bit*`, `shl..rotr`, `lt..ge`)
      (when (i32.and (i32.ge_u $op $DC_SHL_OP) (i32.le_u $op $DC_ROTR_OP))
        (when (i32.ne $t2 $TY_U32) (unreachable))
        (call $emit_scalar_op $fb 2 $op $t1)
        (return $t1))
      (when (i32.ne $t1 $t2) (unreachable))
      (call $emit_scalar_op $fb 2 $op $t1)
      (when (i32.and (i32.ge_u $op $DC_LT_OP) (i32.le_u $op $DC_GE_OP))
        (return $TY_B))
      (return $t1))

    (when (i32.eq $tag $EX_TERNARY)
      (set $op (i32.load offset=4 $ex))
      (set $t1 (call $emit_expr $fctx (i32.load offset=8 $ex) 0))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (when (i32.eqz (call $ty_wasm_vt $t1)) (unreachable))

      (set $t2 (call $emit_expr $fctx (i32.load offset=12 $ex) 0))
      (when (i32.eq $t2 $TY_NEVER) (return $TY_NEVER))
      (when (i32.ne $t2 $TY_U64) (unreachable))

      (set $t3 (call $emit_expr $fctx (i32.load offset=16 $ex) 0))
      (when (i32.eq $t3 $TY_NEVER) (return $TY_NEVER))
      (when (i32.eqz (call $ty_wasm_vt $t3)) (unreachable))

      (when (i32.lt_u $t1 8) (unreachable))
      (set $t (call $buf_get_i32 $type_table $t1))
      (if (i32.eq $op $DC_SEQ_SLICE_OP)
        (then
          (when (i32.or (i32.ne (i32.load $t) $TK_LIST) (i32.ne $t3 $TY_U64))
            (unreachable))
          (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_SEQ_SLICE_OP $DC_STAGEL_LIST_TYPE (i32.load offset=4 $t)))
          (return $t1))
        (else
          (when (i32.load offset=40 $fctx) (unreachable))
          (when (i32.or (i32.ne (i32.load $t) $TK_BUF) (i32.ne (i32.load offset=4 $t) $t3))
            (unreachable))
          (call $fb_op_u32 $fb 0x10 (call $link_seq_op $DC_STAGEL_BUF_SET $DC_STAGEL_BUF_TYPE $t3))
          (return $TY_VOID))))

    (when (i32.eq $tag $EX_CONV)
      (set $t1 (call $emit_expr $fctx (i32.load offset=4 $ex) 0))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (set $cty (call $canon_type $mod_i (i32.load offset=8 $ex)))
      (call $emit_builtin_op
        $fb 1 $DC_CONVERT_OP 3
        $DC_TYPE_CONVERSION
        (call $ty_scalar_fmt_cp $t1)
        (call $ty_scalar_fmt_cp $cty))
      (return $cty))

    (when (i32.eq $tag $EX_BITCAST)
      (set $t1 (call $emit_expr $fctx (i32.load offset=4 $ex) 0))
      (when (i32.eq $t1 $TY_NEVER) (return $TY_NEVER))
      (set $cty (call $canon_type $mod_i (i32.load offset=8 $ex)))
      (call $emit_builtin_op
        $fb 1 $DC_BITCAST_OP 3
        $DC_TYPE_TRANSMUTATION
        (call $ty_scalar_fmt_cp $t1)
        (call $ty_scalar_fmt_cp $cty))
      (return $cty))

    (unreachable))

  (func $emit_routine_body
    (param $mod_idx i32) (param $func_idx i32) (param $r i32)
    (param $is_nested i32) (param $caps_buf i32) (param $inh_scopes i32)
    (local $ret_ty     i32)
    (local $ret_vt     i32)
    (local $fb         i32)
    (local $rest_loc   i32)
    (local $entry_loc  i32)
    (local $root_vars  i32)
    (local $pnames_chk i32)
    (local $cnt        i32)
    (local $i          i32)
    (local $cspec      i32)
    (local $pidx       i32)
    (local $p_nm       i32)
    (local $vb         i32)
    (local $pb         i32)
    (local $prm        i32)
    (local $pty        i32)
    (local $scopes     i32)
    (local $sl         i32)
    (local $fctx       i32)
    (local $body_ty    i32)
    (local $tmp        i32)
    (local $body_p     i32)

    (set $ret_ty (i32.load offset=32 $r))
    (set $ret_vt (call $ty_wasm_vt $ret_ty))
    (set $fb (call $fb_new $ret_vt))

    (set $rest_loc -1)
    (when (i32.and $is_nested $uses_shadow)
      (set $p_nm (call $make_ascii 4 0x7073725F 0)) ;; "_rsp"
      (set $rest_loc (call $fb_add_param $fb $p_nm 4 $FMT_I32)))

    (set $root_vars (call $buf_new 16))
    (when $caps_buf
      (set $cnt (call $buf_count_i32 $caps_buf))
      (set $i 0)
      (block $cdone
        (loop $clp
          (br_if $cdone (i32.ge_u $i $cnt))
          (set $cspec (call $buf_get_i32 $caps_buf $i))
          (set $p_nm (call $make_tag_name 107 $i)) ;; "__k_<i>"
          (set $pidx
            (call $fb_add_param
              $fb $p_nm $ret_len
              (if (result i32) (i32.load offset=12 $cspec)
                (then (i32.const $FMT_I32))
                (else (call $ty_wasm_vt (i32.load offset=8 $cspec))))))
          (set $vb (call $alloc 32))
          (i32.store $vb (i32.load $cspec))
          (i32.store offset=4 $vb (i32.load offset=4 $cspec))
          (i32.store offset=8 $vb (i32.load offset=12 $cspec))
          (i32.store offset=12 $vb $pidx)
          (i32.store offset=16 $vb (i32.load offset=8 $cspec))
          (i32.store offset=20 $vb (i32.load offset=12 $cspec))
          (i32.store offset=24 $vb -1)
          (call $buf_push_i32 $root_vars $vb)
          (set $i (i32.add $i 1))
          (br $clp))))

    (set $pnames_chk (call $buf_new 16))
    (set $pb (i32.load offset=24 $r))
    (set $cnt (call $buf_count_i32 $pb))
    (set $i 0)
    (block $pdone
      (loop $plp
        (br_if $pdone (i32.ge_u $i $cnt))
        (set $prm (call $buf_get_i32 $pb $i))
        (when (i32.eqz (call $nlist_insert $pnames_chk (i32.load offset=4 $prm) (i32.load offset=8 $prm)))
          (unreachable))
        (set $pty (i32.load offset=12 $prm))
        (set $p_nm (call $buf_new 16))
        (call $buf_push_u8 $p_nm 112) ;; 'p'
        (call $buf_push_u8 $p_nm 95)  ;; '_'
        (call $buf_push_sanitized $p_nm (i32.load offset=4 $prm) (i32.load offset=8 $prm))
        (set $pidx (call $fb_add_param $fb (i32.load $p_nm) (i32.load offset=4 $p_nm) (call $ty_wasm_vt $pty)))
        (set $vb (call $alloc 32))
        (i32.store $vb (i32.load offset=4 $prm))
        (i32.store offset=4 $vb (i32.load offset=8 $prm))
        (i32.store offset=8 $vb 0)
        (i32.store offset=12 $vb $pidx)
        (i32.store offset=16 $vb $pty)
        (i32.store offset=20 $vb 0)
        (i32.store offset=24 $vb -1)
        (call $buf_push_i32 $root_vars $vb)
        (set $i (i32.add $i 1))
        (br $plp)))

    (when (i32.and (i32.eqz $is_nested) $has_top_vals)
      (call $fb_op_u32 $fb 0x10 $init_fn_idx))

    (set $entry_loc -1)
    (when $uses_shadow
      (set $entry_loc (call $fb_alloc_local $fb $FMT_I32))
      (call $fb_op_u32 $fb 0x23 $sh_sp_glob)
      (call $fb_op_u32 $fb 0x21 $entry_loc))

    (set $scopes (call $buf_new 16))
    (when $inh_scopes
      (set $cnt (call $buf_count_i32 $inh_scopes))
      (set $i 0)
      (block $idone
        (loop $ilp
          (br_if $idone (i32.ge_u $i $cnt))
          (call $buf_push_i32 $scopes (call $buf_get_i32 $inh_scopes $i))
          (set $i (i32.add $i 1))
          (br $ilp))))
    (set $sl (call $alloc 16))
    (i32.store $sl $root_vars)
    (i32.store offset=4 $sl (call $buf_new 16))
    (call $buf_push_i32 $scopes $sl)

    (set $fctx (call $alloc 48))
    (i32.store $fctx $mod_idx)
    (i32.store offset=4 $fctx (call $alloc_routine_uid))
    (i32.store offset=8 $fctx $is_nested)
    (i32.store offset=12 $fctx $ret_ty)
    (i32.store offset=16 $fctx $entry_loc)
    (i32.store offset=20 $fctx $rest_loc)
    (i32.store offset=24 $fctx $fb)
    (i32.store offset=28 $fctx $scopes)
    (i32.store offset=32 $fctx (call $buf_new 16))
    (i32.store offset=36 $fctx 0)
    (i32.store offset=40 $fctx 0)

    (set $body_ty (call $emit_expr $fctx (i32.load offset=28 $r) 1))
    (when (i32.ne $body_ty $TY_NEVER)
      (when (i32.ne $body_ty $ret_ty) (unreachable))
      (if $ret_vt
        (then
          (set $tmp (call $fb_alloc_local $fb $ret_vt))
          (call $fb_op_u32 $fb 0x21 $tmp)
          (call $emit_routine_shadow_pop $fctx)
          (call $fb_op_u32 $fb 0x20 $tmp))
        (else
          (call $emit_routine_shadow_pop $fctx))))

    (set $body_p (call $fb_finish $fb))
    (call $define_routine $func_idx $body_p $ret_len))

  ;; Walks `$ex` inside module `$mod_idx` and DFS-initializes any local-module `TopVal`
  ;; dependencies before `$tv` itself is evaluated (trapping on intra-module cycles).
  (func $ensure_expr_local_top_deps (param $mod_idx i32) (param $ex i32) (param $bound i32) (param $fb i32) (param $entry_sp_loc i32)
    (local $tag    i32)
    (local $opt_ty i32)
    (local $dep_tv i32)
    (local $elems  i32)
    (local $cnt    i32)
    (local $i      i32)
    (local $sub    i32)
    (local $ibnd   i32)

    (when (i32.eqz $ex) (return))
    (set $tag (i32.load $ex))

    (when (i32.eq $tag $EX_VALUE_REF)
      (when
        (i32.or
          (i32.load offset=8 $ex)
          (i32.eqz (call $nlist_contains $bound (i32.load offset=12 $ex) (i32.load offset=16 $ex))))
        (set $opt_ty 0)
        (when (i32.load offset=4 $ex)
          (set $opt_ty (i32.add 1 (call $canon_type $mod_idx (i32.load offset=4 $ex)))))
        (set $dep_tv
          (call $find_mod_top_val
            $mod_idx 0 $opt_ty
            (i32.load offset=8 $ex) (i32.load offset=12 $ex) (i32.load offset=16 $ex)))
        (when $dep_tv
          (call $emit_top_val_dfs $mod_idx $dep_tv $fb $entry_sp_loc)))
      (return))

    (when (i32.eq $tag $EX_BLOCK)
      (set $ibnd (call $nlist_clone $bound))
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $bdone
        (loop $blp
          (br_if $bdone (i32.ge_u $i $cnt))
          (set $sub (call $buf_get_i32 $elems $i))
          (if (i32.or (i32.eq (i32.load $sub) $EX_LET) (i32.eq (i32.load $sub) $EX_CONST))
            (then
              (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=16 $sub) $ibnd $fb $entry_sp_loc)
              (drop (call $nlist_insert $ibnd (i32.load offset=8 $sub) (i32.load offset=12 $sub))))
            (else
              (call $ensure_expr_local_top_deps $mod_idx $sub $ibnd $fb $entry_sp_loc)))
          (set $i (i32.add $i 1))
          (br $blp)))
      (return))

    (when (i32.or (i32.eq $tag $EX_LIST) (i32.eq $tag $EX_STRUCT_NEW))
      (set $elems (if (result i32) (i32.eq $tag $EX_LIST) (then (i32.load offset=8 $ex)) (else (i32.load offset=16 $ex))))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $ldone
        (loop $llp
          (br_if $ldone (i32.ge_u $i $cnt))
          (call $ensure_expr_local_top_deps $mod_idx (call $buf_get_i32 $elems $i) $bound $fb $entry_sp_loc)
          (set $i (i32.add $i 1))
          (br $llp)))
      (return))

    (when (i32.eq $tag $EX_TUPLE_NEW)
      (set $elems (i32.load offset=4 $ex))
      (set $cnt (call $buf_count_i32 $elems))
      (set $i 0)
      (block $tdone
        (loop $tlp
          (br_if $tdone (i32.ge_u $i $cnt))
          (call $ensure_expr_local_top_deps $mod_idx (call $buf_get_i32 $elems $i) $bound $fb $entry_sp_loc)
          (set $i (i32.add $i 1))
          (br $tlp)))
      (return))

    (when (i32.or (i32.eq $tag $EX_FIELD) (i32.or (i32.eq $tag $EX_CONV) (i32.eq $tag $EX_BITCAST)))
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=4 $ex) $bound $fb $entry_sp_loc)
      (return))
    (when (i32.eq $tag $EX_UNARY)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=8 $ex) $bound $fb $entry_sp_loc)
      (return))
    (when (i32.eq $tag $EX_WITH)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=4 $ex) $bound $fb $entry_sp_loc)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=12 $ex) $bound $fb $entry_sp_loc)
      (return))
    (when (i32.eq $tag $EX_BINARY)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=8 $ex) $bound $fb $entry_sp_loc)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=12 $ex) $bound $fb $entry_sp_loc)
      (return))
    (when (i32.eq $tag $EX_IF)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=4 $ex) $bound $fb $entry_sp_loc)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=8 $ex) $bound $fb $entry_sp_loc)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=12 $ex) $bound $fb $entry_sp_loc)
      (return))
    (when (i32.eq $tag $EX_TERNARY)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=8 $ex) $bound $fb $entry_sp_loc)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=12 $ex) $bound $fb $entry_sp_loc)
      (call $ensure_expr_local_top_deps $mod_idx (i32.load offset=16 $ex) $bound $fb $entry_sp_loc)
      (return)))

  (func $emit_top_val_dfs (param $mod_idx i32) (param $tv i32) (param $fb i32) (param $entry_sp_loc i32)
    (local $state i32)
    (local $fctx  i32)
    (local $ety   i32)
    (set $state (i32.load offset=36 $tv))
    (when (i32.eq $state 2) (return))
    (when (i32.eq $state 1) (unreachable))
    (i32.store offset=36 $tv 1)

    (call $ensure_expr_local_top_deps
      $mod_idx (i32.load offset=24 $tv) (call $buf_new 8) $fb $entry_sp_loc)

    (set $fctx (call $alloc 48))
    (i32.store $fctx $mod_idx)
    (i32.store offset=4 $fctx (call $alloc_routine_uid))
    (i32.store offset=8 $fctx 0)
    (i32.store offset=12 $fctx $TY_VOID)
    (i32.store offset=16 $fctx $entry_sp_loc)
    (i32.store offset=20 $fctx -1)
    (i32.store offset=24 $fctx $fb)
    (i32.store offset=28 $fctx (call $buf_new 8))
    (i32.store offset=32 $fctx (call $buf_new 8))
    (i32.store offset=36 $fctx 0)
    (i32.store offset=40 $fctx 1)

    (set $ety (call $emit_expr $fctx (i32.load offset=24 $tv) 0))
    (when (i32.ne $ety $TY_NEVER)
      (when (i32.ne $ety (i32.load offset=28 $tv)) (unreachable))
      (call $fb_op_u32 $fb 0x24 (i32.load offset=32 $tv)))
    (i32.store offset=36 $tv 2))

  (func $emit_top_values_init_fn
    (local $fb       i32)
    (local $entry_sp i32)
    (local $mcnt     i32)
    (local $mi       i32)
    (local $m        i32)
    (local $vb       i32)
    (local $vcnt     i32)
    (local $vi       i32)
    (local $body_p   i32)

    (when (i32.eqz $has_top_vals) (return))
    (set $fb (call $fb_new 0))

    (call $fb_op_u32 $fb 0x23 $init_flag_glob)
    (call $fb_block $fb 0x04 0)
    (call $fb_op $fb 0x0F)
    (call $fb_op $fb 0x0B)

    (call $fb_i32_const $fb 1)
    (call $fb_op_u32 $fb 0x24 $init_flag_glob)

    (set $entry_sp -1)
    (when $uses_shadow
      (set $entry_sp (call $fb_alloc_local $fb $FMT_I32))
      (call $fb_op_u32 $fb 0x23 $sh_sp_glob)
      (call $fb_op_u32 $fb 0x21 $entry_sp))

    (set $mcnt (call $buf_count_i32 $modules_buf))
    (set $mi 0)
    (block $mdone
      (loop $mlp
        (br_if $mdone (i32.ge_u $mi $mcnt))
        (set $m (call $buf_get_i32 $modules_buf $mi))
        (set $vb (i32.load offset=16 $m))
        (set $vcnt (call $buf_count_i32 $vb))
        (set $vi 0)
        (block $vdone
          (loop $vlp
            (br_if $vdone (i32.ge_u $vi $vcnt))
            (call $emit_top_val_dfs $mi (call $buf_get_i32 $vb $vi) $fb $entry_sp)
            (set $vi (i32.add $vi 1))
            (br $vlp)))
        (set $mi (i32.add $mi 1))
        (br $mlp)))

    (when (i32.ge_s $entry_sp 0)
      (call $fb_op_u32 $fb 0x20 $entry_sp)
      (call $fb_op_u32 $fb 0x24 $sh_sp_glob))

    (set $body_p (call $fb_finish $fb))
    (call $define_routine $init_fn_idx $body_p $ret_len))

  (func $emit_all_module_routines
    (local $mcnt i32)
    (local $mi   i32)
    (local $m    i32)
    (local $rb   i32)
    (local $rcnt i32)
    (local $ri   i32)
    (local $r    i32)
    (set $mcnt (call $buf_count_i32 $modules_buf))
    (set $mi 0)
    (block $mdone
      (loop $mlp
        (br_if $mdone (i32.ge_u $mi $mcnt))
        (set $m (call $buf_get_i32 $modules_buf $mi))
        (set $rb (i32.load offset=20 $m))
        (set $rcnt (call $buf_count_i32 $rb))
        (set $ri 0)
        (block $rdone
          (loop $rlp
            (br_if $rdone (i32.ge_u $ri $rcnt))
            (set $r (call $buf_get_i32 $rb $ri))
            (call $emit_routine_body $mi (i32.load offset=36 $r) $r 0 0 0)
            (set $ri (i32.add $ri 1))
            (br $rlp)))
        (set $mi (i32.add $mi 1))
        (br $mlp))))

  (func $emit_step_entry
    (local $step_lit  i32)
    (local $main_lit  i32)
    (local $last_mi   i32)
    (local $bytes_ty  i32)
    (local $arg_p     i32)
    (local $conv_r    i32)
    (local $step_idx  i32)
    (local $fb        i32)
    (local $alloc_idx i32)
    (local $in_hdr    i32)
    (local $out_hdr   i32)
    (local $in_len    i32)
    (local $body_p    i32)

    (set $step_lit (call $make_ascii 4 0x70657473 0)) ;; "step"
    (set $main_lit (call $make_ascii 4 0x6E69616D 0)) ;; "main"
    (when (i32.ge_s (call $find_routine_by_name $step_lit 4) 0) (return))
    (when (i32.ge_s (call $find_routine_by_name $main_lit 4) 0) (return))

    (set $last_mi (i32.sub (call $buf_count_i32 $modules_buf) 1))
    (set $bytes_ty (call $intern_unary_ty $TK_LIST $TY_U8))
    (set $arg_p (call $alloc 4))
    (i32.store $arg_p $bytes_ty)
    (set $conv_r (call $find_mod_top_rout $last_mi 0 0 $step_lit 4 1 $arg_p))
    (when (i32.and $conv_r (i32.ne (i32.load offset=32 $conv_r) $bytes_ty))
      (set $conv_r 0))

    (set $step_idx (call $reserve_routine $step_lit 4))
    (set $fb (call $fb_new $FMT_I32))
    (when $has_top_vals
      (call $fb_op_u32 $fb 0x10 $init_fn_idx))

    (when $conv_r
      (set $alloc_idx (call $ensure_alloc_helper))
      (set $in_hdr (call $fb_alloc_local $fb $FMT_I32))
      (set $out_hdr (call $fb_alloc_local $fb $FMT_I32))
      (set $in_len (call $fb_alloc_local $fb $FMT_I32))

      (call $fb_i32_const $fb 16)
      (call $fb_op_u32 $fb 0x10 $alloc_idx)
      (call $fb_op_u32 $fb 0x21 $in_hdr)

      (call $fb_i32_const $fb 4)
      (call $fb_mem $fb 0x28 2 0)
      (call $fb_op_u32 $fb 0x21 $in_len)

      (call $fb_op_u32 $fb 0x20 $in_hdr)
      (call $fb_i32_const $fb 0)
      (call $fb_mem $fb 0x28 2 0)
      (call $fb_mem $fb 0x36 2 0)

      (call $fb_op_u32 $fb 0x20 $in_hdr)
      (call $fb_op_u32 $fb 0x20 $in_len)
      (call $fb_mem $fb 0x36 2 4)

      (call $fb_op_u32 $fb 0x20 $in_hdr)
      (call $fb_op_u32 $fb 0x20 $in_len)
      (call $fb_mem $fb 0x36 2 8)

      (call $fb_op_u32 $fb 0x20 $in_hdr)
      (call $fb_i32_const $fb 1)
      (call $fb_mem $fb 0x36 2 12)

      (call $fb_op_u32 $fb 0x20 $in_hdr)
      (call $fb_op_u32 $fb 0x10 (i32.load offset=36 $conv_r))
      (call $fb_op_u32 $fb 0x21 $out_hdr)

      (call $fb_i32_const $fb 8)
      (call $fb_op_u32 $fb 0x20 $out_hdr)
      (call $fb_mem $fb 0x28 2 0)
      (call $fb_mem $fb 0x36 2 0)

      (call $fb_i32_const $fb 12)
      (call $fb_op_u32 $fb 0x20 $out_hdr)
      (call $fb_mem $fb 0x28 2 4)
      (call $fb_mem $fb 0x36 2 0))

    (call $fb_i32_const $fb 0)
    (set $body_p (call $fb_finish $fb))
    (call $define_routine $step_idx $body_p $ret_len))

  (func (export "step") (result i32)
    (local $in_ptr  i32)
    (local $in_end  i32)
    (local $mod_idx i32)
    (local $out_ptr i32)

    (set $in_ptr (i32.load $MB_IN_PTR))
    (set $in_end (i32.add $in_ptr (i32.load $MB_IN_LEN)))
    (when (i32.lt_u $in_end $in_ptr) (unreachable))

    ;; Start compiler bump arena safely past mailbox input
    (global.set $arena_cap (i32.shl (memory.size) 16))
    (global.set $arena_ptr
      (i32.and
        (i32.add (if (result i32) (i32.gt_u $in_end 256) (then (get $in_end)) (else (i32.const 256))) 15)
        -16))
    (global.set $cur_ptr $in_ptr)
    (global.set $cur_end $in_end)

    (global.set $builtin_entries (call $buf_new 64))
    (global.set $out_globals (call $buf_new 32))
    (global.set $out_routines (call $buf_new 64))
    (global.set $alloc_fn_plus1 0)
    (global.set $type_table (call $buf_new 64))
    (global.set $all_structs (call $buf_new 32))
    (global.set $modules_buf (call $buf_new 16))
    (global.set $uses_shadow 0)
    (global.set $has_top_vals 0)
    (global.set $next_rout_uid 1)

    (call $init_types)
    (call $parse_builtin_map)

    ;; Parse one or more `Dc 2192 .. Dc 2193` StageL 3 Token-Dc modules
    (set $mod_idx 0)
    (block $mdone
      (loop $mlp
        (call $skip_comments)
        (br_if $mdone (i32.ge_u $cur_ptr $cur_end))
        (call $buf_push_i32 $modules_buf (call $parse_module $mod_idx))
        (set $mod_idx (i32.add $mod_idx 1))
        (br $mlp)))
    (when (i32.or (i32.eqz $mod_idx) (i32.ne $cur_ptr $cur_end)) (unreachable))

    (call $reserve_runtime_helpers)
    (call $canonicalize_all_declarations)
    (call $emit_shadow_push_helper)
    (call $emit_top_values_init_fn)
    (call $emit_all_module_routines)
    (call $emit_step_entry)

    (set $out_ptr (call $serialize_wasm_module))
    (i32.store $MB_OUT_PTR $out_ptr)
    (i32.store $MB_OUT_LEN $ret_len)
    (i32.const 0))
)
