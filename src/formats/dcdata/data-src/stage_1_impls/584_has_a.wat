;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Stage 1 VSO implementation of Dc 584 ("Has a").
;; Validates a VSO Dc 584 ([atom_dc] | [identifier]) ([value] | [type]) declaration
;; and emits an empty output slice (graph/scope assertion side effect only).
(module
  (memory (export "memory") 1)
  (func (export "_start") (result i32)
    (local i32 i32)
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
    i32.const 8
    local.get 0
    local.get 1
    i32.add
    i32.store align=4 offset=0
    drop
    i32.const 12
    i32.const 0
    i32.store align=4 offset=0
    i32.const 0
  )
)
