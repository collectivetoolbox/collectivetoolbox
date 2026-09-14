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
pub struct SwitchRepeatExprInvalid {
    pub(crate) _root: SharedType<SwitchRepeatExprInvalid>,
    pub(crate) _parent: SharedType<SwitchRepeatExprInvalid>,
    pub(crate) _self_shared: SharedType<Self>,
    codes: RefCell<Vec<u8>>,
    body: RefCell<Vec<SwitchRepeatExprInvalid_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchRepeatExprInvalid_Body {
    SwitchRepeatExprInvalid_One(OptRc<SwitchRepeatExprInvalid_One>),
    SwitchRepeatExprInvalid_Two(OptRc<SwitchRepeatExprInvalid_Two>),
    Bytes(Vec<u8>),
}
impl TryFrom<&SwitchRepeatExprInvalid_Body> for OptRc<SwitchRepeatExprInvalid_One> {
    type Error = KError;
    fn try_from(v: &SwitchRepeatExprInvalid_Body) -> Result<Self, Self::Error> {
        if let SwitchRepeatExprInvalid_Body::SwitchRepeatExprInvalid_One(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchRepeatExprInvalid_One>> for SwitchRepeatExprInvalid_Body {
    fn from(v: OptRc<SwitchRepeatExprInvalid_One>) -> Self {
        Self::SwitchRepeatExprInvalid_One(v)
    }
}
impl TryFrom<&SwitchRepeatExprInvalid_Body> for OptRc<SwitchRepeatExprInvalid_Two> {
    type Error = KError;
    fn try_from(v: &SwitchRepeatExprInvalid_Body) -> Result<Self, Self::Error> {
        if let SwitchRepeatExprInvalid_Body::SwitchRepeatExprInvalid_Two(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchRepeatExprInvalid_Two>> for SwitchRepeatExprInvalid_Body {
    fn from(v: OptRc<SwitchRepeatExprInvalid_Two>) -> Self {
        Self::SwitchRepeatExprInvalid_Two(v)
    }
}
impl TryFrom<&SwitchRepeatExprInvalid_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &SwitchRepeatExprInvalid_Body) -> Result<Self, Self::Error> {
        if let SwitchRepeatExprInvalid_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for SwitchRepeatExprInvalid_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for SwitchRepeatExprInvalid {
    type Root = SwitchRepeatExprInvalid;
    type Parent = SwitchRepeatExprInvalid;

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
        *self_rc.codes.borrow_mut() = Vec::new();
        let l_codes = 3_usize;
        for _i in 0_usize..l_codes {
            self_rc.codes.borrow_mut().push(_io.read_u1()?);
        }
        *self_rc.body.borrow_mut() = Vec::new();
        let l_body = 3_usize;
        for _i in 0_usize..l_body {
            match *(self_rc.codes().get(_i).ok_or(KError::CastError)?) {
                1 => {
                    let _t_body_raw = _io.read_bytes_full()?;
                    let _t_body_raw_io = BytesReader::from(_t_body_raw);
                    let t = Self::read_into::<BytesReader, SwitchRepeatExprInvalid_One>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    self_rc.body.borrow_mut().push(t);
                }
                2 => {
                    let _t_body_raw = _io.read_bytes_full()?;
                    let _t_body_raw_io = BytesReader::from(_t_body_raw);
                    let t = Self::read_into::<BytesReader, SwitchRepeatExprInvalid_Two>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    self_rc.body.borrow_mut().push(t);
                }
                _ => {
                    self_rc.body.borrow_mut().push(_io.read_bytes_full()?.into());
                }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchRepeatExprInvalid {
}
impl SwitchRepeatExprInvalid {
    pub fn codes(&self) -> Ref<'_, Vec<u8>> {
        self.codes.borrow()
    }
}
impl SwitchRepeatExprInvalid {
    pub fn body(&self) -> Ref<'_, Vec<SwitchRepeatExprInvalid_Body>> {
        self.body.borrow()
    }
}
impl SwitchRepeatExprInvalid {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchRepeatExprInvalid {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchRepeatExprInvalid_One {
    pub(crate) _root: SharedType<SwitchRepeatExprInvalid>,
    pub(crate) _parent: SharedType<SwitchRepeatExprInvalid>,
    pub(crate) _self_shared: SharedType<Self>,
    first: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchRepeatExprInvalid_One {
    type Root = SwitchRepeatExprInvalid;
    type Parent = SwitchRepeatExprInvalid;

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
        *self_rc.first.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchRepeatExprInvalid_One {
}
impl SwitchRepeatExprInvalid_One {
    pub fn first(&self) -> Ref<'_, Vec<u8>> {
        self.first.borrow()
    }
}
impl SwitchRepeatExprInvalid_One {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchRepeatExprInvalid_Two {
    pub(crate) _root: SharedType<SwitchRepeatExprInvalid>,
    pub(crate) _parent: SharedType<SwitchRepeatExprInvalid>,
    pub(crate) _self_shared: SharedType<Self>,
    second: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchRepeatExprInvalid_Two {
    type Root = SwitchRepeatExprInvalid;
    type Parent = SwitchRepeatExprInvalid;

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
        *self_rc.second.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchRepeatExprInvalid_Two {
}
impl SwitchRepeatExprInvalid_Two {
    pub fn second(&self) -> Ref<'_, Vec<u8>> {
        self.second.borrow()
    }
}
impl SwitchRepeatExprInvalid_Two {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
