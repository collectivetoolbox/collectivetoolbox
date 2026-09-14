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
pub struct NavParentSwitch {
    pub(crate) _root: SharedType<NavParentSwitch>,
    pub(crate) _parent: SharedType<NavParentSwitch>,
    pub(crate) _self_shared: SharedType<Self>,
    category: RefCell<u8>,
    content: RefCell<Option<NavParentSwitch_Content>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum NavParentSwitch_Content {
    NavParentSwitch_Element1(OptRc<NavParentSwitch_Element1>),
}
impl From<&NavParentSwitch_Content> for OptRc<NavParentSwitch_Element1> {
    fn from(v: &NavParentSwitch_Content) -> Self {
        let NavParentSwitch_Content::NavParentSwitch_Element1(x) = v;
        x.clone()
    }
}
impl TryFrom<&NavParentSwitch_Content> for OptRc<NavParentSwitch_Element1> {
    type Error = KError;
    fn try_from(v: &NavParentSwitch_Content) -> Result<Self, Self::Error> {
        if let NavParentSwitch_Content::NavParentSwitch_Element1(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<NavParentSwitch_Element1>> for NavParentSwitch_Content {
    fn from(v: OptRc<NavParentSwitch_Element1>) -> Self {
        Self::NavParentSwitch_Element1(v)
    }
}
impl KStruct for NavParentSwitch {
    type Root = NavParentSwitch;
    type Parent = NavParentSwitch;

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
        *self_rc.category.borrow_mut() = _io.read_u1()?;
        match *self_rc.category() {
            1 => {
                let t = Self::read_into::<_, NavParentSwitch_Element1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitch {
}
impl NavParentSwitch {
    pub fn category(&self) -> Ref<'_, u8> {
        self.category.borrow()
    }
}
impl NavParentSwitch {
    pub fn content(&self) -> Ref<'_, Option<NavParentSwitch_Content>> {
        self.content.borrow()
    }
}
impl NavParentSwitch {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentSwitch_Element1 {
    pub(crate) _root: SharedType<NavParentSwitch>,
    pub(crate) _parent: SharedType<NavParentSwitch>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u8>,
    subelement: RefCell<OptRc<NavParentSwitch_Subelement1>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentSwitch_Element1 {
    type Root = NavParentSwitch;
    type Parent = NavParentSwitch;

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
        *self_rc.foo.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, NavParentSwitch_Subelement1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.subelement.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitch_Element1 {
}
impl NavParentSwitch_Element1 {
    pub fn foo(&self) -> Ref<'_, u8> {
        self.foo.borrow()
    }
}
impl NavParentSwitch_Element1 {
    pub fn subelement(&self) -> Ref<'_, OptRc<NavParentSwitch_Subelement1>> {
        self.subelement.borrow()
    }
}
impl NavParentSwitch_Element1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentSwitch_Subelement1 {
    pub(crate) _root: SharedType<NavParentSwitch>,
    pub(crate) _parent: SharedType<NavParentSwitch_Element1>,
    pub(crate) _self_shared: SharedType<Self>,
    bar: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentSwitch_Subelement1 {
    type Root = NavParentSwitch;
    type Parent = NavParentSwitch_Element1;

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
        if ((to_i128(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.foo())) == (to_i128(66))) {
            *self_rc.bar.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentSwitch_Subelement1 {
}
impl NavParentSwitch_Subelement1 {
    pub fn bar(&self) -> Ref<'_, u8> {
        self.bar.borrow()
    }
}
impl NavParentSwitch_Subelement1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
