;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 1937 ("Collective Toolbox Workspace").
(module
  (memory (export "memory") 1)
  (func (export "_start") (result i32)
    (local i32 i32 i32 i32 i32)
    i32.const 0
    i32.load align=4 offset=0
    local.tee 0
    i32.const 4
    i32.load align=4 offset=0
    i32.add
    local.tee 1
    local.set 4
    i32.const 8
    local.get 4
    i32.store align=4 offset=0
    local.get 0
    local.get 1
    i32.eq
    if
      i32.const 64
      i32.const 1
      i32.store align=4 offset=0
      i32.const 68
      i64.const 1000000
      i64.store align=1 offset=0
      i32.const 76
      i64.const 256
      i64.store align=1 offset=0
      i32.const 84
      i64.const -1
      i64.store align=1 offset=0
      i32.const 92
      i64.const 65536
      i64.store align=1 offset=0
      i32.const 100
      i64.const -1
      i64.store align=1 offset=0
      i32.const 108
      i64.const 4194304
      i64.store align=1 offset=0
      i32.const 116
      i64.const 16777216
      i64.store align=1 offset=0
      i32.const 124
      i64.const 1048576
      i64.store align=1 offset=0
      i32.const 132
      i64.const 16777216
      i64.store align=1 offset=0
    else
      i32.const 0
      local.set 2
      block
        loop
          local.get 0
          local.get 1
          i32.eq
          br_if 1
          local.get 2
          i32.const 76
          i32.ge_u
          if
            unreachable
          end
          local.get 0
          i32.load8_u align=1 offset=0
          local.tee 3
          i32.const 128
          i32.lt_u
          if
            local.get 0
            i32.const 1
            i32.add
            local.set 0
          else
            local.get 0
            i32.const 2
            i32.add
            local.get 1
            i32.gt_u
            if
              unreachable
            end
            local.get 3
            i32.const 254
            i32.and
            i32.const 194
            i32.ne
            if
              unreachable
            end
            local.get 3
            i32.const 3
            i32.and
            i32.const 6
            i32.shl
            local.get 0
            i32.load8_u align=1 offset=1
            i32.const 63
            i32.and
            i32.or
            local.set 3
            local.get 0
            i32.const 2
            i32.add
            local.set 0
          end
          local.get 2
          local.get 3
          i32.store8 align=1 offset=64
          local.get 2
          i32.const 1
          i32.add
          local.set 2
          br 0
        end
      end
      local.get 2
      i32.const 76
      i32.ne
      if
        unreachable
      end
    end
    i32.const 64
    i32.load align=4 offset=0
    i32.const 1
    i32.ne
    if
      unreachable
    end
    i32.const 68
    i64.load align=1 offset=0
    i64.eqz
    if
      unreachable
    end
    i32.const 76
    i64.load align=1 offset=0
    i64.eqz
    if
      unreachable
    end
    i32.const 0
    local.set 2
    block
      loop
        local.get 2
        i32.const 76
        i32.eq
        br_if 1
        local.get 2
        i32.load8_u align=1 offset=64
        local.tee 3
        i32.const 128
        i32.lt_u
        if
          local.get 4
          local.get 3
          i32.store8 align=1 offset=0
          local.get 4
          i32.const 1
          i32.add
          local.set 4
        else
          local.get 4
          local.get 3
          i32.const 6
          i32.shr_u
          i32.const 192
          i32.or
          i32.store8 align=1 offset=0
          local.get 4
          local.get 3
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 align=1 offset=1
          local.get 4
          i32.const 2
          i32.add
          local.set 4
        end
        local.get 2
        i32.const 1
        i32.add
        local.set 2
        br 0
      end
    end
    i32.const 12
    local.get 4
    local.get 1
    i32.sub
    i32.store align=4 offset=0
    i32.const 0
  )
)
