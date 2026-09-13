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
pub struct InstanceInRepeatUntil {
    pub(crate) _root: SharedType<InstanceInRepeatUntil>,
    pub(crate) _parent: SharedType<InstanceInRepeatUntil>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<i16>>,
    _io: RefCell<BytesReader>,
    f_until_val: Cell<bool>,
    until_val: RefCell<i16>,
}
impl KStruct for InstanceInRepeatUntil {
    type Root = InstanceInRepeatUntil;
    type Parent = InstanceInRepeatUntil;

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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                self_rc.entries.borrow_mut().push(_io.read_s2le()?);
                let _t_entries = self_rc.entries.borrow();
                let Some(_tmpa) = _t_entries.last() else { break; };
                let _tmpa = *_tmpa;
                _i = _i.saturating_add(1);
                if _tmpa == *self_rc.until_val()? { break; }
            }
        }
        Ok(())
    }
}
impl InstanceInRepeatUntil {
    pub fn until_val(
        &self
    ) -> KResult<Ref<'_, i16>> {
        let _io = self._io.borrow();
        if self.f_until_val.get() {
            return Ok(self.until_val.borrow());
        }
        self.f_until_val.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((_io.pos()).saturating_add(12_usize))?)?;
        *self.until_val.borrow_mut() = _io.read_s2le()?;
        _io.seek(_pos)?;
        Ok(self.until_val.borrow())
    }
}
impl InstanceInRepeatUntil {
    pub fn entries(&self) -> Ref<'_, Vec<i16>> {
        self.entries.borrow()
    }
}
impl InstanceInRepeatUntil {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
