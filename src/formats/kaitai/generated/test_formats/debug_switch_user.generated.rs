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
pub struct DebugSwitchUser {
    pub(crate) _root: SharedType<DebugSwitchUser>,
    pub(crate) _parent: SharedType<DebugSwitchUser>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    data: RefCell<Option<DebugSwitchUser_Data>>,
    _io: RefCell<BytesReader>,
    data_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum DebugSwitchUser_Data {
    DebugSwitchUser_One(OptRc<DebugSwitchUser_One>),
    DebugSwitchUser_Two(OptRc<DebugSwitchUser_Two>),
}
impl TryFrom<&DebugSwitchUser_Data> for OptRc<DebugSwitchUser_One> {
    type Error = KError;
    fn try_from(v: &DebugSwitchUser_Data) -> Result<Self, Self::Error> {
        if let DebugSwitchUser_Data::DebugSwitchUser_One(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<DebugSwitchUser_One>> for DebugSwitchUser_Data {
    fn from(v: OptRc<DebugSwitchUser_One>) -> Self {
        Self::DebugSwitchUser_One(v)
    }
}
impl TryFrom<&DebugSwitchUser_Data> for OptRc<DebugSwitchUser_Two> {
    type Error = KError;
    fn try_from(v: &DebugSwitchUser_Data) -> Result<Self, Self::Error> {
        if let DebugSwitchUser_Data::DebugSwitchUser_Two(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<DebugSwitchUser_Two>> for DebugSwitchUser_Data {
    fn from(v: OptRc<DebugSwitchUser_Two>) -> Self {
        Self::DebugSwitchUser_Two(v)
    }
}
impl KStruct for DebugSwitchUser {
    type Root = DebugSwitchUser;
    type Parent = DebugSwitchUser;

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
        *self_rc.code.borrow_mut() = _io.read_u1()?;
        match *self_rc.code() {
            1 => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, DebugSwitchUser_One>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            2 => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, DebugSwitchUser_Two>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugSwitchUser {
}
impl DebugSwitchUser {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl DebugSwitchUser {
    pub fn data(&self) -> Ref<'_, Option<DebugSwitchUser_Data>> {
        self.data.borrow()
    }
}
impl DebugSwitchUser {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DebugSwitchUser {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DebugSwitchUser_One {
    pub(crate) _root: SharedType<DebugSwitchUser>,
    pub(crate) _parent: SharedType<DebugSwitchUser>,
    pub(crate) _self_shared: SharedType<Self>,
    val: RefCell<i16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DebugSwitchUser_One {
    type Root = DebugSwitchUser;
    type Parent = DebugSwitchUser;

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
        *self_rc.val.borrow_mut() = _io.read_s2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugSwitchUser_One {
}
impl DebugSwitchUser_One {
    pub fn val(&self) -> Ref<'_, i16> {
        self.val.borrow()
    }
}
impl DebugSwitchUser_One {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DebugSwitchUser_Two {
    pub(crate) _root: SharedType<DebugSwitchUser>,
    pub(crate) _parent: SharedType<DebugSwitchUser>,
    pub(crate) _self_shared: SharedType<Self>,
    val: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DebugSwitchUser_Two {
    type Root = DebugSwitchUser;
    type Parent = DebugSwitchUser;

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
        *self_rc.val.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugSwitchUser_Two {
}
impl DebugSwitchUser_Two {
    pub fn val(&self) -> Ref<'_, u16> {
        self.val.borrow()
    }
}
impl DebugSwitchUser_Two {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
