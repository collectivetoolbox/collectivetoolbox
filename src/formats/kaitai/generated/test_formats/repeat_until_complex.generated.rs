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
pub struct RepeatUntilComplex {
    pub(crate) _root: SharedType<RepeatUntilComplex>,
    pub(crate) _parent: SharedType<RepeatUntilComplex>,
    pub(crate) _self_shared: SharedType<Self>,
    first: RefCell<Vec<OptRc<RepeatUntilComplex_TypeU1>>>,
    second: RefCell<Vec<OptRc<RepeatUntilComplex_TypeU2>>>,
    third: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RepeatUntilComplex {
    type Root = RepeatUntilComplex;
    type Parent = RepeatUntilComplex;

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
        *self_rc.first.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, RepeatUntilComplex_TypeU1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.first.borrow_mut().push(t);
                let _t_first = self_rc.first.borrow();
                let Some(_tmpa) = _t_first.last() else { break; };
                _i = _i.saturating_add(1);
                if ((to_i128(*_tmpa.count())) == (to_i128(0))) { break; }
            }
        }
        *self_rc.second.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, RepeatUntilComplex_TypeU2>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.second.borrow_mut().push(t);
                let _t_second = self_rc.second.borrow();
                let Some(_tmpa) = _t_second.last() else { break; };
                _i = _i.saturating_add(1);
                if ((to_i128(*_tmpa.count())) == (to_i128(0))) { break; }
            }
        }
        *self_rc.third.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                self_rc.third.borrow_mut().push(_io.read_u1()?);
                let _t_third = self_rc.third.borrow();
                let Some(_tmpa) = _t_third.last() else { break; };
                let _tmpa = *_tmpa;
                _i = _i.saturating_add(1);
                if ((to_i128(_tmpa)) == (to_i128(0))) { break; }
            }
        }
        Ok(())
    }
}
impl RepeatUntilComplex {
}
impl RepeatUntilComplex {
    pub fn first(&self) -> Ref<'_, Vec<OptRc<RepeatUntilComplex_TypeU1>>> {
        self.first.borrow()
    }
}
impl RepeatUntilComplex {
    pub fn second(&self) -> Ref<'_, Vec<OptRc<RepeatUntilComplex_TypeU2>>> {
        self.second.borrow()
    }
}
impl RepeatUntilComplex {
    pub fn third(&self) -> Ref<'_, Vec<u8>> {
        self.third.borrow()
    }
}
impl RepeatUntilComplex {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RepeatUntilComplex_TypeU1 {
    pub(crate) _root: SharedType<RepeatUntilComplex>,
    pub(crate) _parent: SharedType<RepeatUntilComplex>,
    pub(crate) _self_shared: SharedType<Self>,
    count: RefCell<u8>,
    values: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RepeatUntilComplex_TypeU1 {
    type Root = RepeatUntilComplex;
    type Parent = RepeatUntilComplex;

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
        *self_rc.count.borrow_mut() = _io.read_u1()?;
        *self_rc.values.borrow_mut() = Vec::new();
        let l_values = usize::from(*self_rc.count());
        for _i in 0_usize..l_values {
            self_rc.values.borrow_mut().push(_io.read_u1()?);
        }
        Ok(())
    }
}
impl RepeatUntilComplex_TypeU1 {
}
impl RepeatUntilComplex_TypeU1 {
    pub fn count(&self) -> Ref<'_, u8> {
        self.count.borrow()
    }
}
impl RepeatUntilComplex_TypeU1 {
    pub fn values(&self) -> Ref<'_, Vec<u8>> {
        self.values.borrow()
    }
}
impl RepeatUntilComplex_TypeU1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RepeatUntilComplex_TypeU2 {
    pub(crate) _root: SharedType<RepeatUntilComplex>,
    pub(crate) _parent: SharedType<RepeatUntilComplex>,
    pub(crate) _self_shared: SharedType<Self>,
    count: RefCell<u16>,
    values: RefCell<Vec<u16>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RepeatUntilComplex_TypeU2 {
    type Root = RepeatUntilComplex;
    type Parent = RepeatUntilComplex;

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
        *self_rc.count.borrow_mut() = _io.read_u2le()?;
        *self_rc.values.borrow_mut() = Vec::new();
        let l_values = usize::from(*self_rc.count());
        for _i in 0_usize..l_values {
            self_rc.values.borrow_mut().push(_io.read_u2le()?);
        }
        Ok(())
    }
}
impl RepeatUntilComplex_TypeU2 {
}
impl RepeatUntilComplex_TypeU2 {
    pub fn count(&self) -> Ref<'_, u16> {
        self.count.borrow()
    }
}
impl RepeatUntilComplex_TypeU2 {
    pub fn values(&self) -> Ref<'_, Vec<u16>> {
        self.values.borrow()
    }
}
impl RepeatUntilComplex_TypeU2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
