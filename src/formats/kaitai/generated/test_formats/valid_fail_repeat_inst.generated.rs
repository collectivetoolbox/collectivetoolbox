// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/*
== License information for parts derived from kaitai_struct_tests, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

MIT License

Copyright (c) 2019 Kaitai Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


*/
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct ValidFailRepeatInst {
    pub(crate) _root: SharedType<ValidFailRepeatInst>,
    pub(crate) _parent: SharedType<ValidFailRepeatInst>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    a_raw: RefCell<Vec<u8>>,
    f_inst: Cell<bool>,
    inst: RefCell<Vec<u32>>,
}
impl KStruct for ValidFailRepeatInst {
    type Root = ValidFailRepeatInst;
    type Parent = ValidFailRepeatInst;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
    fn read<S: KStream>(
        self_rc: &OptRc<Self>,
        io: &S,
        root: SharedType<Self::Root>,
        parent: SharedType<Self::Parent>,
    ) -> KResult<()> {
        *self_rc._io.borrow_mut() = io.clone();
        self_rc._root.set(root.get());
        self_rc._parent.set(parent.get());
        self_rc._self_shared.set(Ok(self_rc.clone()));
        let _io = io;
        if self_rc.inst()?.len() == 0 {
            *self_rc.a.borrow_mut() = _io.read_bytes(0_usize)?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ValidFailRepeatInst {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn inst(
        &self
    ) -> KResult<Ref<'_, Vec<u32>>> {
        let _io = self._io.borrow();
        if self.f_inst.get() {
            return Ok(self.inst.borrow());
        }
        self.f_inst.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.inst.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                self.inst.borrow_mut().push(_io.read_u4be()?);
                _i = _i.saturating_add(1);
            }
        }
        let expected: u32 = (305419896).try_into()?;
        if !self.inst.borrow().iter().all(|_x| *_x == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/instances/inst".to_string() }));
        }
        _io.seek(_pos)?;
        Ok(self.inst.borrow())
    }
}
impl ValidFailRepeatInst {
    pub fn a(&self) -> Ref<'_, Vec<u8>> {
        self.a.borrow()
    }
}
impl ValidFailRepeatInst {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ValidFailRepeatInst {
    pub fn a_raw(&self) -> Ref<'_, Vec<u8>> {
        self.a_raw.borrow()
    }
}
