;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 260 ("Begin literal value").
;; Validates a Dc 260 .. Dc 261 literal container, strips its leading type header
;; (either a Dc 262 .. Dc 263 compound type definition or a single type token) and
;; trailing Dc 261 closer, unescapes Dc 255 once in the payload, and writes the
;; unescaped payload bytes to the output slice.
(module
  (memory (export "memory") 1)
  (func (export "_start") (result i32)
    (local i32 i32 i32 i32 i32 i32)
    i32.const 4
    i32.load align=4 offset=0
    local.tee 1
    i32.const 12
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
    i32.const 4
    i32.sub
    local.set 1
    i32.const 8
    local.get 3
    i32.store align=4 offset=0
    ;; Verify leading Dc 260 (-2062498764) and trailing Dc 261 (-2045721548)
    local.get 0
    i32.load align=1 offset=0
    i32.const -2062498764
    i32.ne
    if
      unreachable
    end
    local.get 1
    i32.load align=1 offset=0
    i32.const -2045721548
    i32.ne
    if
      unreachable
    end
    local.get 0
    i32.const 4
    i32.add
    local.set 0
    ;; Skip leading type header: either Dc 262 (-2028944332) .. Dc 263 (-2012167116) or 1 DcTF token
    local.get 0
    i32.load align=1 offset=0
    i32.const -2028944332
    i32.eq
    if
      i32.const 1
      local.set 5
      local.get 0
      i32.const 4
      i32.add
      local.set 0
      block
        loop
          local.get 5
          i32.eqz
          br_if 1
          local.get 0
          i32.const 4
          i32.add
          local.get 1
          i32.gt_u
          if
            unreachable
          end
          local.get 0
          i32.load align=1 offset=0
          i32.const -2028944332
          i32.eq
          if
            local.get 5
            i32.const 1
            i32.add
            local.set 5
          else
            local.get 0
            i32.load align=1 offset=0
            i32.const -2012167116
            i32.eq
            if
              local.get 5
              i32.const 1
              i32.sub
              local.set 5
            end
          end
          local.get 0
          i32.const 4
          i32.add
          local.set 0
          br 0
        end
      end
    else
      local.get 0
      i32.load8_u align=1 offset=0
      local.tee 4
      i32.const 128
      i32.lt_u
      if (result i32)
        i32.const 1
      else
        local.get 4
        i32.const 255
        i32.lt_u
        if (result i32)
          i32.const 255
          local.get 4
          i32.sub
          i32.const 24
          i32.shl
          i32.clz
        else
          i32.const 8
          local.get 0
          i32.load8_u align=1 offset=1
          i32.const 2
          i32.shr_u
          i32.const 15
          i32.and
          i32.add
        end
      end
      local.get 0
      i32.add
      local.set 0
    end
    ;; Unescape payload up to local 1 (closing Dc 261)
    block
      loop
        local.get 0
        local.get 1
        i32.eq
        br_if 1
        local.get 0
        local.get 1
        i32.gt_u
        if
          unreachable
        end
        local.get 0
        i32.const 4
        i32.add
        local.get 1
        i32.le_u
        if
          local.get 0
          i32.load align=1 offset=0
          i32.const -2045721548
          i32.eq
          if
            unreachable
          end
          local.get 0
          i32.load align=1 offset=0
          i32.const -1081896716
          i32.eq
          if
            local.get 0
            i32.const 4
            i32.add
            local.tee 0
            local.get 1
            i32.ge_u
            if
              unreachable
            end
          end
        end
        local.get 0
        i32.load8_u align=1 offset=0
        local.tee 4
        i32.const 128
        i32.lt_u
        if (result i32)
          i32.const 1
        else
          local.get 4
          i32.const 255
          i32.lt_u
          if (result i32)
            i32.const 255
            local.get 4
            i32.sub
            i32.const 24
            i32.shl
            i32.clz
          else
            i32.const 8
            local.get 0
            i32.load8_u align=1 offset=1
            i32.const 2
            i32.shr_u
            i32.const 15
            i32.and
            i32.add
          end
        end
        local.set 5
        local.get 0
        local.get 5
        i32.add
        local.get 1
        i32.gt_u
        if
          unreachable
        end
        block
          loop
            local.get 5
            i32.eqz
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
            local.get 5
            i32.const 1
            i32.sub
            local.set 5
            br 0
          end
        end
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
