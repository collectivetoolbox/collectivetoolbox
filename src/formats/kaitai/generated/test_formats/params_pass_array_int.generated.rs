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
pub struct ParamsPassArrayInt {
    pub(crate) _root: SharedType<ParamsPassArrayInt>,
    pub(crate) _parent: SharedType<ParamsPassArrayInt>,
    pub(crate) _self_shared: SharedType<Self>,
    ints: RefCell<Vec<u16>>,
    pass_ints: RefCell<OptRc<ParamsPassArrayInt_WantsInts>>,
    pass_ints_calc: RefCell<OptRc<ParamsPassArrayInt_WantsInts>>,
    _io: RefCell<BytesReader>,
    f_ints_calc: Cell<bool>,
    ints_calc: RefCell<i32>,
}
impl KStruct for ParamsPassArrayInt {
    type Root = ParamsPassArrayInt;
    type Parent = ParamsPassArrayInt;

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
        *self_rc.ints.borrow_mut() = Vec::new();
        let l_ints = 3_usize;
        for _i in 0_usize..l_ints {
            self_rc.ints.borrow_mut().push(_io.read_u2le()?);
        }
        let f = |t : &mut ParamsPassArrayInt_WantsInts| Ok(t.set_params(self_rc.ints().clone()));
        let t = Self::read_into_with_init::<_, ParamsPassArrayInt_WantsInts>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.pass_ints.borrow_mut() = t;
        let f = |t : &mut ParamsPassArrayInt_WantsInts| Ok(t.set_params((*self_rc.ints_calc()?).try_into().map_err(|_| KError::CastError)?));
        let t = Self::read_into_with_init::<_, ParamsPassArrayInt_WantsInts>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.pass_ints_calc.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassArrayInt {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn ints_calc(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_ints_calc.get() {
            return Ok(self.ints_calc.borrow());
        }
        self.f_ints_calc.set(true);
        *self.ints_calc.borrow_mut() = (u2[]::try_from(vec![27643, 7])?).try_into()?;
        Ok(self.ints_calc.borrow())
    }
}
impl ParamsPassArrayInt {
    pub fn ints(&self) -> Ref<'_, Vec<u16>> {
        self.ints.borrow()
    }
}
impl ParamsPassArrayInt {
    pub fn pass_ints(&self) -> Ref<'_, OptRc<ParamsPassArrayInt_WantsInts>> {
        self.pass_ints.borrow()
    }
}
impl ParamsPassArrayInt {
    pub fn pass_ints_calc(&self) -> Ref<'_, OptRc<ParamsPassArrayInt_WantsInts>> {
        self.pass_ints_calc.borrow()
    }
}
impl ParamsPassArrayInt {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassArrayInt_WantsInts {
    pub(crate) _root: SharedType<ParamsPassArrayInt>,
    pub(crate) _parent: SharedType<ParamsPassArrayInt>,
    pub(crate) _self_shared: SharedType<Self>,
    nums: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassArrayInt_WantsInts {
    type Root = ParamsPassArrayInt;
    type Parent = ParamsPassArrayInt;

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
impl ParamsPassArrayInt_WantsInts {
    pub fn nums(&self) -> Ref<'_, u16> {
        self.nums.borrow()
    }
}
impl ParamsPassArrayInt_WantsInts {
    pub fn set_params(&mut self, nums: u16) {
        *self.nums.borrow_mut() = nums;
    }
}
impl ParamsPassArrayInt_WantsInts {
}
impl ParamsPassArrayInt_WantsInts {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
