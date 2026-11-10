;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 0 implementation of Dc 541 ("Enter identifier scope").
;; Validates a Dc 541 .. Dc 542 lexical scope container and emits its interior span.
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
    i32.const 4
    i32.sub
    local.set 1
    i32.const 8
    local.get 3
    i32.store align=4 offset=0
    local.get 0
    i32.const 4
    i32.add
    local.set 0
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
