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
pub struct EofExceptionSwitchUser {
    pub(crate) _root: SharedType<EofExceptionSwitchUser>,
    pub(crate) _parent: SharedType<EofExceptionSwitchUser>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u16>,
    data: RefCell<Option<EofExceptionSwitchUser_Data>>,
    _io: RefCell<BytesReader>,
    data_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum EofExceptionSwitchUser_Data {
    EofExceptionSwitchUser_Two(OptRc<EofExceptionSwitchUser_Two>),
    EofExceptionSwitchUser_One(OptRc<EofExceptionSwitchUser_One>),
}
impl TryFrom<&EofExceptionSwitchUser_Data> for OptRc<EofExceptionSwitchUser_Two> {
    type Error = KError;
    fn try_from(v: &EofExceptionSwitchUser_Data) -> Result<Self, Self::Error> {
        if let EofExceptionSwitchUser_Data::EofExceptionSwitchUser_Two(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<EofExceptionSwitchUser_Two>> for EofExceptionSwitchUser_Data {
    fn from(v: OptRc<EofExceptionSwitchUser_Two>) -> Self {
        Self::EofExceptionSwitchUser_Two(v)
    }
}
impl TryFrom<&EofExceptionSwitchUser_Data> for OptRc<EofExceptionSwitchUser_One> {
    type Error = KError;
    fn try_from(v: &EofExceptionSwitchUser_Data) -> Result<Self, Self::Error> {
        if let EofExceptionSwitchUser_Data::EofExceptionSwitchUser_One(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<EofExceptionSwitchUser_One>> for EofExceptionSwitchUser_Data {
    fn from(v: OptRc<EofExceptionSwitchUser_One>) -> Self {
        Self::EofExceptionSwitchUser_One(v)
    }
}
impl KStruct for EofExceptionSwitchUser {
    type Root = EofExceptionSwitchUser;
    type Parent = EofExceptionSwitchUser;

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
        *self_rc.code.borrow_mut() = _io.read_u2le()?;
        match *self_rc.code() {
            2 => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, EofExceptionSwitchUser_Two>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            511 => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, EofExceptionSwitchUser_One>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EofExceptionSwitchUser {
}
impl EofExceptionSwitchUser {
    pub fn code(&self) -> Ref<'_, u16> {
        self.code.borrow()
    }
}
impl EofExceptionSwitchUser {
    pub fn data(&self) -> Ref<'_, Option<EofExceptionSwitchUser_Data>> {
        self.data.borrow()
    }
}
impl EofExceptionSwitchUser {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl EofExceptionSwitchUser {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EofExceptionSwitchUser_One {
    pub(crate) _root: SharedType<EofExceptionSwitchUser>,
    pub(crate) _parent: SharedType<EofExceptionSwitchUser>,
    pub(crate) _self_shared: SharedType<Self>,
    val: RefCell<i16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EofExceptionSwitchUser_One {
    type Root = EofExceptionSwitchUser;
    type Parent = EofExceptionSwitchUser;

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
impl EofExceptionSwitchUser_One {
}
impl EofExceptionSwitchUser_One {
    pub fn val(&self) -> Ref<'_, i16> {
        self.val.borrow()
    }
}
impl EofExceptionSwitchUser_One {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EofExceptionSwitchUser_Two {
    pub(crate) _root: SharedType<EofExceptionSwitchUser>,
    pub(crate) _parent: SharedType<EofExceptionSwitchUser>,
    pub(crate) _self_shared: SharedType<Self>,
    val: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EofExceptionSwitchUser_Two {
    type Root = EofExceptionSwitchUser;
    type Parent = EofExceptionSwitchUser;

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
impl EofExceptionSwitchUser_Two {
}
impl EofExceptionSwitchUser_Two {
    pub fn val(&self) -> Ref<'_, u16> {
        self.val.borrow()
    }
}
impl EofExceptionSwitchUser_Two {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
