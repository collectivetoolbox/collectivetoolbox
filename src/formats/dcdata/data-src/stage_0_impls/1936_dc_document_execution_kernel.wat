;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 1936 ("Dc document execution kernel").
;;
;; Reads an optional 76-byte execution configuration record from the linear-memory
;; mailbox (or synthesizes the canonical default configuration when the input slice
;; is empty), validates the header version and nonzero fuel/call-depth budgets, and
;; writes the validated 76-byte configuration record to the output mailbox.
(module
  (memory (export "memory") 1)

  ;; Linear-memory mailbox header offsets (bytes 0..16)
  (const $MB_IN_PTR  0)
  (const $MB_IN_LEN  4)
  (const $MB_OUT_PTR 8)
  (const $MB_OUT_LEN 12)

  ;; Scratch configuration buffer layout at byte offset 64 (76 bytes total)
  (const $CFG_BASE            64)
  (const $CFG_LEN             76)
  (const $CFG_VERSION         64)
  (const $CFG_FUEL_BUDGET     68)
  (const $CFG_CALL_DEPTH      76)
  (const $CFG_MAX_NODES       84)
  (const $CFG_MEMORY_PAGES    92)
  (const $CFG_WALL_TIME_MS    100)
  (const $CFG_IPC_QUOTA       108)
  (const $CFG_MAX_OUTPUT      116)
  (const $CFG_MAX_GRAPH_BYTES 124)
  (const $CFG_ARENA_CAP       132)

  (func (export "_start") (result i32)
    (local $in_ptr  i32)
    (local $in_end  i32)
    (local $idx     i32)
    (local $byte    i32)
    (local $out_ptr i32)

    ;; Initialize $in_ptr, $in_end = $in_ptr + $in_len, and $out_ptr = $in_end.
    (set $out_ptr
      (tee $in_end
        (i32.add
          (tee $in_ptr (i32.load $MB_IN_PTR))
          (i32.load $MB_IN_LEN))))
    (i32.store $MB_OUT_PTR $out_ptr)

    (if (i32.eq $in_ptr $in_end)
      (then
        ;; Empty input: populate default 76-byte kernel configuration record.
        (i32.store $CFG_VERSION 1)
        (i64.store align=1 $CFG_FUEL_BUDGET     1000000)
        (i64.store align=1 $CFG_CALL_DEPTH      256)
        (i64.store align=1 $CFG_MAX_NODES       -1)
        (i64.store align=1 $CFG_MEMORY_PAGES    65536)
        (i64.store align=1 $CFG_WALL_TIME_MS    -1)
        (i64.store align=1 $CFG_IPC_QUOTA       4194304)
        (i64.store align=1 $CFG_MAX_OUTPUT      16777216)
        (i64.store align=1 $CFG_MAX_GRAPH_BYTES 1048576)
        (i64.store align=1 $CFG_ARENA_CAP       16777216))
      (else
        ;; Decode UTF-8 / Latin-1 byte stream from [$in_ptr .. $in_end) into $CFG_BASE.
        (set $idx 0)
        (block $decode_done
          (loop $decode_loop
            (br_if $decode_done (i32.eq $in_ptr $in_end))
            (when (i32.ge_u $idx $CFG_LEN) (unreachable))
            (if (i32.lt_u (tee $byte (i32.load8_u $in_ptr)) 128)
              (then
                (set $in_ptr (i32.add $in_ptr 1)))
              (else
                (when (i32.gt_u (i32.add $in_ptr 2) $in_end) (unreachable))
                (when (i32.ne (i32.and $byte 254) 194) (unreachable))
                (set $byte
                  (i32.or
                    (i32.shl (i32.and $byte 3) 6)
                    (i32.and (i32.load8_u offset=1 $in_ptr) 63)))
                (set $in_ptr (i32.add $in_ptr 2))))
            (i32.store8 offset=$CFG_BASE $idx $byte)
            (set $idx (i32.add $idx 1))
            (br $decode_loop)))
        (when (i32.ne $idx $CFG_LEN) (unreachable))))

    ;; Validate configuration invariants (version == 1, fuel > 0, call_depth > 0).
    (when (i32.ne (i32.load $CFG_VERSION) 1) (unreachable))
    (when (i64.eqz (i64.load align=1 $CFG_FUEL_BUDGET)) (unreachable))
    (when (i64.eqz (i64.load align=1 $CFG_CALL_DEPTH)) (unreachable))

    ;; Encode validated 76-byte configuration record to the output slice at $out_ptr.
    (set $idx 0)
    (block $encode_done
      (loop $encode_loop
        (br_if $encode_done (i32.eq $idx $CFG_LEN))
        (if (i32.lt_u (tee $byte (i32.load8_u offset=$CFG_BASE $idx)) 128)
          (then
            (i32.store8 $out_ptr $byte)
            (set $out_ptr (i32.add $out_ptr 1)))
          (else
            (i32.store8 $out_ptr (i32.or (i32.shr_u $byte 6) 192))
            (i32.store8 offset=1 $out_ptr (i32.or (i32.and $byte 63) 128))
            (set $out_ptr (i32.add $out_ptr 2))))
        (set $idx (i32.add $idx 1))
        (br $encode_loop)))

    (i32.store $MB_OUT_LEN (i32.sub $out_ptr $in_end))
    (i32.const 0)
  )
)
