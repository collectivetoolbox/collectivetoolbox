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
pub struct InstanceIoUser {
    pub(crate) _root: SharedType<InstanceIoUser>,
    pub(crate) _parent: SharedType<InstanceIoUser>,
    pub(crate) _self_shared: SharedType<Self>,
    qty_entries: RefCell<u32>,
    entries: RefCell<Vec<OptRc<InstanceIoUser_Entry>>>,
    strings: RefCell<OptRc<InstanceIoUser_StringsObj>>,
    _io: RefCell<BytesReader>,
    strings_raw: RefCell<Vec<u8>>,
}
impl KStruct for InstanceIoUser {
    type Root = InstanceIoUser;
    type Parent = InstanceIoUser;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.qty_entries.borrow_mut() = _io.read_u4le()?;
        *self_rc.entries.borrow_mut() = Vec::new();
        let l_entries = usize::try_from(*self_rc.qty_entries())?;
        for _i in 0_usize..l_entries {
            let t = Self::read_into::<_, InstanceIoUser_Entry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.entries.borrow_mut().push(t);
        }
        let _raw_strings = _io.read_bytes_full()?;
        *self_rc.strings_raw.borrow_mut() = _raw_strings.clone();
        let _io_strings = BytesReader::from(_raw_strings);
        let t = Self::read_into::<BytesReader, InstanceIoUser_StringsObj>(&_io_strings, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.strings.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUser {
}
impl InstanceIoUser {
    pub fn qty_entries(&self) -> Ref<'_, u32> {
        self.qty_entries.borrow()
    }
}
impl InstanceIoUser {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<InstanceIoUser_Entry>>> {
        self.entries.borrow()
    }
}
impl InstanceIoUser {
    pub fn strings(&self) -> Ref<'_, OptRc<InstanceIoUser_StringsObj>> {
        self.strings.borrow()
    }
}
impl InstanceIoUser {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceIoUser {
    pub fn strings_raw(&self) -> Ref<'_, Vec<u8>> {
        self.strings_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceIoUser_Entry {
    pub(crate) _root: SharedType<InstanceIoUser>,
    pub(crate) _parent: SharedType<InstanceIoUser>,
    pub(crate) _self_shared: SharedType<Self>,
    name_ofs: RefCell<u32>,
    value: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_name: Cell<bool>,
    name: RefCell<String>,
}
impl KStruct for InstanceIoUser_Entry {
    type Root = InstanceIoUser;
    type Parent = InstanceIoUser;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.name_ofs.borrow_mut() = _io.read_u4le()?;
        *self_rc.value.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUser_Entry {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn name(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_name.get() {
            return Ok(self.name.borrow());
        }
        self.f_name.set(true);
        let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.strings()._io());
        let _pos = io.pos();
        io.seek(usize::try_from(*self.name_ofs())?)?;
        *self.name.borrow_mut() = bytes_to_str(&io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        io.seek(_pos)?;
        Ok(self.name.borrow())
    }
}
impl InstanceIoUser_Entry {
    pub fn name_ofs(&self) -> Ref<'_, u32> {
        self.name_ofs.borrow()
    }
}
impl InstanceIoUser_Entry {
    pub fn value(&self) -> Ref<'_, u32> {
        self.value.borrow()
    }
}
impl InstanceIoUser_Entry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceIoUser_StringsObj {
    pub(crate) _root: SharedType<InstanceIoUser>,
    pub(crate) _parent: SharedType<InstanceIoUser>,
    pub(crate) _self_shared: SharedType<Self>,
    str: RefCell<Vec<String>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for InstanceIoUser_StringsObj {
    type Root = InstanceIoUser;
    type Parent = InstanceIoUser;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.str.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                self_rc.str.borrow_mut().push(bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUser_StringsObj {
}
impl InstanceIoUser_StringsObj {
    pub fn str(&self) -> Ref<'_, Vec<String>> {
        self.str.borrow()
    }
}
impl InstanceIoUser_StringsObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
