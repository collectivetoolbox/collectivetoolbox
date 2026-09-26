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
pub struct IoLocalVar {
    pub(crate) _root: SharedType<IoLocalVar>,
    pub(crate) _parent: SharedType<IoLocalVar>,
    pub(crate) _self_shared: SharedType<Self>,
    skip: RefCell<Vec<u8>>,
    always_null: RefCell<u8>,
    followup: RefCell<u8>,
    _io: RefCell<BytesReader>,
    skip_raw: RefCell<Vec<u8>>,
    mess_up_raw: RefCell<Vec<u8>>,
    f_mess_up: Cell<bool>,
    mess_up: RefCell<Option<IoLocalVar_MessUp>>,
}
#[derive(Debug, Clone)]
pub enum IoLocalVar_MessUp {
    IoLocalVar_Dummy(OptRc<IoLocalVar_Dummy>),
    Bytes(Vec<u8>),
}
impl TryFrom<&IoLocalVar_MessUp> for OptRc<IoLocalVar_Dummy> {
    type Error = KError;
    fn try_from(v: &IoLocalVar_MessUp) -> Result<Self, Self::Error> {
        if let IoLocalVar_MessUp::IoLocalVar_Dummy(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<IoLocalVar_Dummy>> for IoLocalVar_MessUp {
    fn from(v: OptRc<IoLocalVar_Dummy>) -> Self {
        Self::IoLocalVar_Dummy(v)
    }
}
impl TryFrom<&IoLocalVar_MessUp> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &IoLocalVar_MessUp) -> Result<Self, Self::Error> {
        if let IoLocalVar_MessUp::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for IoLocalVar_MessUp {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for IoLocalVar {
    type Root = IoLocalVar;
    type Parent = IoLocalVar;

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
        *self_rc.skip.borrow_mut() = _io.read_bytes(20_usize)?;
        if OptRc::<IoLocalVar_Dummy>::try_from(&*(self_rc.mess_up()?).as_ref().ok_or(KError::CastError)?)?._io().pos() < 0 {
            *self_rc.always_null.borrow_mut() = _io.read_u1()?;
        }
        *self_rc.followup.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IoLocalVar {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn mess_up(
        &self
    ) -> KResult<Ref<'_, Option<IoLocalVar_MessUp>>> {
        let _io = self._io.borrow();
        if self.f_mess_up.get() {
            return Ok(self.mess_up.borrow());
        }
        self.f_mess_up.set(true);
        let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
        let _pos = io.pos();
        io.seek(8_usize)?;
        match 2 {
            1 => {
                *self.mess_up_raw.borrow_mut() = io.read_bytes(2_usize)?.into();
                let mess_up_raw = self.mess_up_raw.borrow();
                let _t_mess_up_raw_io = BytesReader::from(mess_up_raw.clone());
                let t = Self::read_into::<BytesReader, IoLocalVar_Dummy>(&_t_mess_up_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.mess_up.borrow_mut() = Some(t);
            }
            2 => {
                *self.mess_up_raw.borrow_mut() = io.read_bytes(2_usize)?.into();
                let mess_up_raw = self.mess_up_raw.borrow();
                let _t_mess_up_raw_io = BytesReader::from(mess_up_raw.clone());
                let t = Self::read_into::<BytesReader, IoLocalVar_Dummy>(&_t_mess_up_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.mess_up.borrow_mut() = Some(t);
            }
            _ => {
                *self.mess_up.borrow_mut() = Some(io.read_bytes(2_usize)?.into());
            }
        }
        io.seek(_pos)?;
        Ok(self.mess_up.borrow())
    }
}
impl IoLocalVar {
    pub fn skip(&self) -> Ref<'_, Vec<u8>> {
        self.skip.borrow()
    }
}
impl IoLocalVar {
    pub fn always_null(&self) -> Ref<'_, u8> {
        self.always_null.borrow()
    }
}
impl IoLocalVar {
    pub fn followup(&self) -> Ref<'_, u8> {
        self.followup.borrow()
    }
}
impl IoLocalVar {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl IoLocalVar {
    pub fn skip_raw(&self) -> Ref<'_, Vec<u8>> {
        self.skip_raw.borrow()
    }
}
impl IoLocalVar {
    pub fn mess_up_raw(&self) -> Ref<'_, Vec<u8>> {
        self.mess_up_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IoLocalVar_Dummy {
    pub(crate) _root: SharedType<IoLocalVar>,
    pub(crate) _parent: SharedType<IoLocalVar>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IoLocalVar_Dummy {
    type Root = IoLocalVar;
    type Parent = IoLocalVar;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IoLocalVar_Dummy {
}
impl IoLocalVar_Dummy {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
