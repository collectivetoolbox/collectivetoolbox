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
pub struct SwitchRepeatExpr {
    pub(crate) _root: SharedType<SwitchRepeatExpr>,
    pub(crate) _parent: SharedType<SwitchRepeatExpr>,
    pub(crate) _self_shared: SharedType<Self>,
    codes: RefCell<Vec<u8>>,
    body: RefCell<Vec<SwitchRepeatExpr_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchRepeatExpr_Body {
    SwitchRepeatExpr_One(OptRc<SwitchRepeatExpr_One>),
    SwitchRepeatExpr_Two(OptRc<SwitchRepeatExpr_Two>),
    Bytes(Vec<u8>),
}
impl TryFrom<&SwitchRepeatExpr_Body> for OptRc<SwitchRepeatExpr_One> {
    type Error = KError;
    fn try_from(v: &SwitchRepeatExpr_Body) -> Result<Self, Self::Error> {
        if let SwitchRepeatExpr_Body::SwitchRepeatExpr_One(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchRepeatExpr_One>> for SwitchRepeatExpr_Body {
    fn from(v: OptRc<SwitchRepeatExpr_One>) -> Self {
        Self::SwitchRepeatExpr_One(v)
    }
}
impl TryFrom<&SwitchRepeatExpr_Body> for OptRc<SwitchRepeatExpr_Two> {
    type Error = KError;
    fn try_from(v: &SwitchRepeatExpr_Body) -> Result<Self, Self::Error> {
        if let SwitchRepeatExpr_Body::SwitchRepeatExpr_Two(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchRepeatExpr_Two>> for SwitchRepeatExpr_Body {
    fn from(v: OptRc<SwitchRepeatExpr_Two>) -> Self {
        Self::SwitchRepeatExpr_Two(v)
    }
}
impl TryFrom<&SwitchRepeatExpr_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &SwitchRepeatExpr_Body) -> Result<Self, Self::Error> {
        if let SwitchRepeatExpr_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for SwitchRepeatExpr_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for SwitchRepeatExpr {
    type Root = SwitchRepeatExpr;
    type Parent = SwitchRepeatExpr;

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
                    let t = Self::read_into::<_, SwitchRepeatExpr_One>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    self_rc.body.borrow_mut().push(t);
                }
                2 => {
                    let t = Self::read_into::<_, SwitchRepeatExpr_One>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    self_rc.body.borrow_mut().push(t);
                }
                7 => {
                    let t = Self::read_into::<_, SwitchRepeatExpr_Two>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
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
impl SwitchRepeatExpr {
}
impl SwitchRepeatExpr {
    pub fn codes(&self) -> Ref<'_, Vec<u8>> {
        self.codes.borrow()
    }
}
impl SwitchRepeatExpr {
    pub fn body(&self) -> Ref<'_, Vec<SwitchRepeatExpr_Body>> {
        self.body.borrow()
    }
}
impl SwitchRepeatExpr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchRepeatExpr {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchRepeatExpr_One {
    pub(crate) _root: SharedType<SwitchRepeatExpr>,
    pub(crate) _parent: SharedType<SwitchRepeatExpr>,
    pub(crate) _self_shared: SharedType<Self>,
    first: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchRepeatExpr_One {
    type Root = SwitchRepeatExpr;
    type Parent = SwitchRepeatExpr;

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
impl SwitchRepeatExpr_One {
}
impl SwitchRepeatExpr_One {
    pub fn first(&self) -> Ref<'_, Vec<u8>> {
        self.first.borrow()
    }
}
impl SwitchRepeatExpr_One {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchRepeatExpr_Two {
    pub(crate) _root: SharedType<SwitchRepeatExpr>,
    pub(crate) _parent: SharedType<SwitchRepeatExpr>,
    pub(crate) _self_shared: SharedType<Self>,
    second: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchRepeatExpr_Two {
    type Root = SwitchRepeatExpr;
    type Parent = SwitchRepeatExpr;

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
impl SwitchRepeatExpr_Two {
}
impl SwitchRepeatExpr_Two {
    pub fn second(&self) -> Ref<'_, Vec<u8>> {
        self.second.borrow()
    }
}
impl SwitchRepeatExpr_Two {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
