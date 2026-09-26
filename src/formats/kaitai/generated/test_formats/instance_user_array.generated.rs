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
pub struct InstanceUserArray {
    pub(crate) _root: SharedType<InstanceUserArray>,
    pub(crate) _parent: SharedType<InstanceUserArray>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs: RefCell<u32>,
    entry_size: RefCell<u32>,
    qty_entries: RefCell<u32>,
    _io: RefCell<BytesReader>,
    user_entries_raw: RefCell<Vec<Vec<u8>>>,
    f_user_entries: Cell<bool>,
    user_entries: RefCell<Vec<OptRc<InstanceUserArray_Entry>>>,
}
impl KStruct for InstanceUserArray {
    type Root = InstanceUserArray;
    type Parent = InstanceUserArray;

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
        *self_rc.ofs.borrow_mut() = _io.read_u4le()?;
        *self_rc.entry_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.qty_entries.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceUserArray {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn user_entries(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<InstanceUserArray_Entry>>>> {
        let _io = self._io.borrow();
        if self.f_user_entries.get() {
            return Ok(self.user_entries.borrow());
        }
        self.f_user_entries.set(true);
        if ((to_i128(*self.ofs())) > (to_i128(0))) {
            let _pos = _io.pos();
            _io.seek(usize::try_from(*self.ofs())?)?;
            *self.user_entries_raw.borrow_mut() = Vec::new();
            *self.user_entries.borrow_mut() = Vec::new();
            let l_user_entries = usize::try_from(*self.qty_entries())?;
            for _i in 0_usize..l_user_entries {
                self.user_entries_raw.borrow_mut().push(_io.read_bytes(usize::try_from(*self.entry_size())?)?.into());
                let user_entries_raw = self.user_entries_raw.borrow();
                let _io_user_entries_raw = BytesReader::from(user_entries_raw.last().ok_or(KError::EmptyIterator)?.clone());
                let t = Self::read_into::<BytesReader, InstanceUserArray_Entry>(&_io_user_entries_raw, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                self.user_entries.borrow_mut().push(t);
            }
            _io.seek(_pos)?;
        }
        Ok(self.user_entries.borrow())
    }
}
impl InstanceUserArray {
    pub fn ofs(&self) -> Ref<'_, u32> {
        self.ofs.borrow()
    }
}
impl InstanceUserArray {
    pub fn entry_size(&self) -> Ref<'_, u32> {
        self.entry_size.borrow()
    }
}
impl InstanceUserArray {
    pub fn qty_entries(&self) -> Ref<'_, u32> {
        self.qty_entries.borrow()
    }
}
impl InstanceUserArray {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceUserArray {
    pub fn user_entries_raw(&self) -> Ref<'_, Vec<Vec<u8>>> {
        self.user_entries_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceUserArray_Entry {
    pub(crate) _root: SharedType<InstanceUserArray>,
    pub(crate) _parent: SharedType<InstanceUserArray>,
    pub(crate) _self_shared: SharedType<Self>,
    word1: RefCell<u16>,
    word2: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for InstanceUserArray_Entry {
    type Root = InstanceUserArray;
    type Parent = InstanceUserArray;

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
        *self_rc.word1.borrow_mut() = _io.read_u2le()?;
        *self_rc.word2.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceUserArray_Entry {
}
impl InstanceUserArray_Entry {
    pub fn word1(&self) -> Ref<'_, u16> {
        self.word1.borrow()
    }
}
impl InstanceUserArray_Entry {
    pub fn word2(&self) -> Ref<'_, u16> {
        self.word2.borrow()
    }
}
impl InstanceUserArray_Entry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
