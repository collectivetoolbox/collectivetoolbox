;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 307 ("void lang.invoke([Routine] routine | [Routine name] routineName)").
;; Strips the leading Dc 307 token and optional Dc 258 .. Dc 259 parameter wrapper,
;; emitting the invoked target routine or identifier span.
(module
  (memory (export "memory") 1)
  (func (export "_start") (result i32)
    (local i32 i32 i32 i32)
    i32.const 4
    i32.load align=4 offset=0
    local.tee 1
    i32.const 8
    i32.lt_u
    if
      unreachable
    end
    i32.const 0
    i32.load align=4 offset=0
    local.tee 0
    local.get 1
    i32.add
    local.tee 3
    local.set 2
    local.get 3
    local.set 1
    i32.const 8
    local.get 3
    i32.store align=4 offset=0
    ;; Advance past leading Dc 307 (4-byte DcTF token)
    local.get 0
    i32.const 4
    i32.add
    local.set 0
    ;; If wrapped in Dc 258 (-2096053196) .. Dc 259 (-2079275980), strip wrapper
    local.get 0
    i32.const 8
    i32.add
    local.get 1
    i32.le_u
    if
      local.get 0
      i32.load align=1 offset=0
      i32.const -2096053196
      i32.eq
      if
        local.get 1
        i32.const 4
        i32.sub
        i32.load align=1 offset=0
        i32.const -2079275980
        i32.ne
        if
          unreachable
        end
        local.get 0
        i32.const 4
        i32.add
        local.set 0
        local.get 1
        i32.const 4
        i32.sub
        local.set 1
      end
    end
    block
      loop
        local.get 0
        local.get 1
        i32.eq
        br_if 1
        local.get 2
        local.get 0
        i32.load8_u align=1 offset=0
        i32.store8 align=1 offset=0
        local.get 0
        i32.const 1
        i32.add
        local.set 0
        local.get 2
        i32.const 1
        i32.add
        local.set 2
        br 0
      end
    end
    i32.const 12
    local.get 2
    local.get 3
    i32.sub
    i32.store align=4 offset=0
    i32.const 0
  )
)
