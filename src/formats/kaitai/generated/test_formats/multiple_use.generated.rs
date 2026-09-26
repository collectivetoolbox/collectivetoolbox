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
pub struct MultipleUse {
    pub(crate) _root: SharedType<MultipleUse>,
    pub(crate) _parent: SharedType<MultipleUse>,
    pub(crate) _self_shared: SharedType<Self>,
    t1: RefCell<OptRc<MultipleUse_Type1>>,
    t2: RefCell<OptRc<MultipleUse_Type2>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MultipleUse {
    type Root = MultipleUse;
    type Parent = MultipleUse;

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
        let t = Self::read_into::<_, MultipleUse_Type1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.t1.borrow_mut() = t;
        let t = Self::read_into::<_, MultipleUse_Type2>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.t2.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MultipleUse {
}
impl MultipleUse {
    pub fn t1(&self) -> Ref<'_, OptRc<MultipleUse_Type1>> {
        self.t1.borrow()
    }
}
impl MultipleUse {
    pub fn t2(&self) -> Ref<'_, OptRc<MultipleUse_Type2>> {
        self.t2.borrow()
    }
}
impl MultipleUse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MultipleUse_Multi {
    pub(crate) _root: SharedType<MultipleUse>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<i32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MultipleUse_Multi {
    type Root = MultipleUse;
    type Parent = KStructUnit;

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
        *self_rc.value.borrow_mut() = _io.read_s4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MultipleUse_Multi {
}
impl MultipleUse_Multi {
    pub fn value(&self) -> Ref<'_, i32> {
        self.value.borrow()
    }
}
impl MultipleUse_Multi {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MultipleUse_Type1 {
    pub(crate) _root: SharedType<MultipleUse>,
    pub(crate) _parent: SharedType<MultipleUse>,
    pub(crate) _self_shared: SharedType<Self>,
    first_use: RefCell<OptRc<MultipleUse_Multi>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MultipleUse_Type1 {
    type Root = MultipleUse;
    type Parent = MultipleUse;

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
        let t = Self::read_into::<_, MultipleUse_Multi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.first_use.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MultipleUse_Type1 {
}
impl MultipleUse_Type1 {
    pub fn first_use(&self) -> Ref<'_, OptRc<MultipleUse_Multi>> {
        self.first_use.borrow()
    }
}
impl MultipleUse_Type1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MultipleUse_Type2 {
    pub(crate) _root: SharedType<MultipleUse>,
    pub(crate) _parent: SharedType<MultipleUse>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_second_use: Cell<bool>,
    second_use: RefCell<OptRc<MultipleUse_Multi>>,
}
impl KStruct for MultipleUse_Type2 {
    type Root = MultipleUse;
    type Parent = MultipleUse;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MultipleUse_Type2 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn second_use(
        &self
    ) -> KResult<Ref<'_, OptRc<MultipleUse_Multi>>> {
        let _io = self._io.borrow();
        if self.f_second_use.get() {
            return Ok(self.second_use.borrow());
        }
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        let t = Self::read_into::<_, MultipleUse_Multi>(&*_io, Some(self._root.clone()), None)?.into();
        *self.second_use.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.second_use.borrow())
    }
}
impl MultipleUse_Type2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
