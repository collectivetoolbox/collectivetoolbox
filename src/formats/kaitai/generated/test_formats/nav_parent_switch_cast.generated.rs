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
pub struct NavParentSwitchCast {
    pub(crate) _root: SharedType<NavParentSwitchCast>,
    pub(crate) _parent: SharedType<NavParentSwitchCast>,
    pub(crate) _self_shared: SharedType<Self>,
    main: RefCell<OptRc<NavParentSwitchCast_Foo>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentSwitchCast {
    type Root = NavParentSwitchCast;
    type Parent = NavParentSwitchCast;

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
        let t = Self::read_into::<_, NavParentSwitchCast_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitchCast {
}
impl NavParentSwitchCast {
    pub fn main(&self) -> Ref<'_, OptRc<NavParentSwitchCast_Foo>> {
        self.main.borrow()
    }
}
impl NavParentSwitchCast {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentSwitchCast_Foo {
    pub(crate) _root: SharedType<NavParentSwitchCast>,
    pub(crate) _parent: SharedType<NavParentSwitchCast>,
    pub(crate) _self_shared: SharedType<Self>,
    buf_type: RefCell<u8>,
    flag: RefCell<u8>,
    buf: RefCell<Option<NavParentSwitchCast_Foo_Buf>>,
    _io: RefCell<BytesReader>,
    buf_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum NavParentSwitchCast_Foo_Buf {
    NavParentSwitchCast_Foo_Zero(OptRc<NavParentSwitchCast_Foo_Zero>),
    NavParentSwitchCast_Foo_One(OptRc<NavParentSwitchCast_Foo_One>),
    Bytes(Vec<u8>),
}
impl TryFrom<&NavParentSwitchCast_Foo_Buf> for OptRc<NavParentSwitchCast_Foo_Zero> {
    type Error = KError;
    fn try_from(v: &NavParentSwitchCast_Foo_Buf) -> Result<Self, Self::Error> {
        if let NavParentSwitchCast_Foo_Buf::NavParentSwitchCast_Foo_Zero(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<NavParentSwitchCast_Foo_Zero>> for NavParentSwitchCast_Foo_Buf {
    fn from(v: OptRc<NavParentSwitchCast_Foo_Zero>) -> Self {
        Self::NavParentSwitchCast_Foo_Zero(v)
    }
}
impl TryFrom<&NavParentSwitchCast_Foo_Buf> for OptRc<NavParentSwitchCast_Foo_One> {
    type Error = KError;
    fn try_from(v: &NavParentSwitchCast_Foo_Buf) -> Result<Self, Self::Error> {
        if let NavParentSwitchCast_Foo_Buf::NavParentSwitchCast_Foo_One(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<NavParentSwitchCast_Foo_One>> for NavParentSwitchCast_Foo_Buf {
    fn from(v: OptRc<NavParentSwitchCast_Foo_One>) -> Self {
        Self::NavParentSwitchCast_Foo_One(v)
    }
}
impl TryFrom<&NavParentSwitchCast_Foo_Buf> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &NavParentSwitchCast_Foo_Buf) -> Result<Self, Self::Error> {
        if let NavParentSwitchCast_Foo_Buf::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for NavParentSwitchCast_Foo_Buf {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for NavParentSwitchCast_Foo {
    type Root = NavParentSwitchCast;
    type Parent = NavParentSwitchCast;

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
        *self_rc.buf_type.borrow_mut() = _io.read_u1()?;
        *self_rc.flag.borrow_mut() = _io.read_u1()?;
        match *self_rc.buf_type() {
            0 => {
                *self_rc.buf_raw.borrow_mut() = _io.read_bytes(4_usize)?.into();
                let buf_raw = self_rc.buf_raw.borrow();
                let _t_buf_raw_io = BytesReader::from(buf_raw.clone());
                let t = Self::read_into::<BytesReader, NavParentSwitchCast_Foo_Zero>(&_t_buf_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.buf.borrow_mut() = Some(t);
            }
            1 => {
                *self_rc.buf_raw.borrow_mut() = _io.read_bytes(4_usize)?.into();
                let buf_raw = self_rc.buf_raw.borrow();
                let _t_buf_raw_io = BytesReader::from(buf_raw.clone());
                let t = Self::read_into::<BytesReader, NavParentSwitchCast_Foo_One>(&_t_buf_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.buf.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.buf.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitchCast_Foo {
}
impl NavParentSwitchCast_Foo {
    pub fn buf_type(&self) -> Ref<'_, u8> {
        self.buf_type.borrow()
    }
}
impl NavParentSwitchCast_Foo {
    pub fn flag(&self) -> Ref<'_, u8> {
        self.flag.borrow()
    }
}
impl NavParentSwitchCast_Foo {
    pub fn buf(&self) -> Ref<'_, Option<NavParentSwitchCast_Foo_Buf>> {
        self.buf.borrow()
    }
}
impl NavParentSwitchCast_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl NavParentSwitchCast_Foo {
    pub fn buf_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentSwitchCast_Foo_Common {
    pub(crate) _root: SharedType<NavParentSwitchCast>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_flag: Cell<bool>,
    flag: RefCell<u8>,
}
impl KStruct for NavParentSwitchCast_Foo_Common {
    type Root = NavParentSwitchCast;
    type Parent = KStructUnit;

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
impl NavParentSwitchCast_Foo_Common {
    pub fn flag(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_flag.get() {
            return Ok(self.flag.borrow());
        }
        self.f_flag.set(true);
        *self.flag.borrow_mut() = (*OptRc::<NavParentSwitchCast_Foo>::try_from(&self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?)?.flag()).try_into()?;
        Ok(self.flag.borrow())
    }
}
impl NavParentSwitchCast_Foo_Common {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentSwitchCast_Foo_One {
    pub(crate) _root: SharedType<NavParentSwitchCast>,
    pub(crate) _parent: SharedType<NavParentSwitchCast_Foo>,
    pub(crate) _self_shared: SharedType<Self>,
    branch: RefCell<OptRc<NavParentSwitchCast_Foo_Common>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentSwitchCast_Foo_One {
    type Root = NavParentSwitchCast;
    type Parent = NavParentSwitchCast_Foo;

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
        let t = Self::read_into::<_, NavParentSwitchCast_Foo_Common>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.branch.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitchCast_Foo_One {
}
impl NavParentSwitchCast_Foo_One {
    pub fn branch(&self) -> Ref<'_, OptRc<NavParentSwitchCast_Foo_Common>> {
        self.branch.borrow()
    }
}
impl NavParentSwitchCast_Foo_One {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentSwitchCast_Foo_Zero {
    pub(crate) _root: SharedType<NavParentSwitchCast>,
    pub(crate) _parent: SharedType<NavParentSwitchCast_Foo>,
    pub(crate) _self_shared: SharedType<Self>,
    branch: RefCell<OptRc<NavParentSwitchCast_Foo_Common>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentSwitchCast_Foo_Zero {
    type Root = NavParentSwitchCast;
    type Parent = NavParentSwitchCast_Foo;

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
        let t = Self::read_into::<_, NavParentSwitchCast_Foo_Common>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.branch.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitchCast_Foo_Zero {
}
impl NavParentSwitchCast_Foo_Zero {
    pub fn branch(&self) -> Ref<'_, OptRc<NavParentSwitchCast_Foo_Common>> {
        self.branch.borrow()
    }
}
impl NavParentSwitchCast_Foo_Zero {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
