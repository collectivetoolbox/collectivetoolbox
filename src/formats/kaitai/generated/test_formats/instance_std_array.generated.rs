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
pub struct InstanceStdArray {
    pub(crate) _root: SharedType<InstanceStdArray>,
    pub(crate) _parent: SharedType<InstanceStdArray>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs: RefCell<u32>,
    entry_size: RefCell<u32>,
    qty_entries: RefCell<u32>,
    _io: RefCell<BytesReader>,
    entries_raw: RefCell<Vec<Vec<u8>>>,
    f_entries: Cell<bool>,
    entries: RefCell<Vec<Vec<u8>>>,
}
impl KStruct for InstanceStdArray {
    type Root = InstanceStdArray;
    type Parent = InstanceStdArray;

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
        *self_rc.ofs.borrow_mut() = _io.read_u4le()?;
        *self_rc.entry_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.qty_entries.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceStdArray {
    pub fn entries(
        &self
    ) -> KResult<Ref<'_, Vec<Vec<u8>>>> {
        let _io = self._io.borrow();
        if self.f_entries.get() {
            return Ok(self.entries.borrow());
        }
        self.f_entries.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs())?)?;
        *self.entries_raw.borrow_mut() = Vec::new();
        *self.entries.borrow_mut() = Vec::new();
        let l_entries = usize::try_from(*self.qty_entries())?;
        for _i in 0_usize..l_entries {
            self.entries_raw.borrow_mut().push(_io.read_bytes(usize::try_from(*self.entry_size())?)?.into());
            let entries_raw = self.entries_raw.borrow();
            let _io_entries_raw = BytesReader::from(entries_raw.last().ok_or(KError::EmptyIterator)?.clone());
            self.entries.borrow_mut().push(_io_entries_raw.read_bytes(usize::try_from(*self.entry_size())?)?);
        }
        _io.seek(_pos)?;
        Ok(self.entries.borrow())
    }
}
impl InstanceStdArray {
    pub fn ofs(&self) -> Ref<'_, u32> {
        self.ofs.borrow()
    }
}
impl InstanceStdArray {
    pub fn entry_size(&self) -> Ref<'_, u32> {
        self.entry_size.borrow()
    }
}
impl InstanceStdArray {
    pub fn qty_entries(&self) -> Ref<'_, u32> {
        self.qty_entries.borrow()
    }
}
impl InstanceStdArray {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceStdArray {
    pub fn entries_raw(&self) -> Ref<'_, Vec<Vec<u8>>> {
        self.entries_raw.borrow()
    }
}
