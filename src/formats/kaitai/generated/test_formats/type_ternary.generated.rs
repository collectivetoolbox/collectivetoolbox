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
pub struct TypeTernary {
    pub(crate) _root: SharedType<TypeTernary>,
    pub(crate) _parent: SharedType<TypeTernary>,
    pub(crate) _self_shared: SharedType<Self>,
    dif_wo_hack: RefCell<OptRc<TypeTernary_Dummy>>,
    dif_with_hack: RefCell<OptRc<TypeTernary_Dummy>>,
    _io: RefCell<BytesReader>,
    dif_wo_hack_raw: RefCell<Vec<u8>>,
    dif_with_hack_raw: RefCell<Vec<u8>>,
    f_dif: Cell<bool>,
    dif: RefCell<OptRc<TypeTernary_Dummy>>,
    f_dif_value: Cell<bool>,
    dif_value: RefCell<u8>,
    f_is_hack: Cell<bool>,
    is_hack: RefCell<bool>,
}
impl KStruct for TypeTernary {
    type Root = TypeTernary;
    type Parent = TypeTernary;

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
        if !(*self_rc.is_hack()?) {
            let _raw_dif_wo_hack = _io.read_bytes(1_usize)?;
            *self_rc.dif_wo_hack_raw.borrow_mut() = _raw_dif_wo_hack.clone();
            let _io_dif_wo_hack = BytesReader::from(_raw_dif_wo_hack);
            let t = Self::read_into::<BytesReader, TypeTernary_Dummy>(&_io_dif_wo_hack, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.dif_wo_hack.borrow_mut() = t;
        }
        let _raw_dif_with_hack = _io.read_bytes(1_usize)?;
        *self_rc.dif_with_hack_raw.borrow_mut() = _raw_dif_with_hack.clone();
        let _processed_dif_with_hack = process_xor_one(&_raw_dif_with_hack, 3_u8);
        let _io_dif_with_hack = BytesReader::from(_processed_dif_with_hack);
        let t = Self::read_into::<BytesReader, TypeTernary_Dummy>(&_io_dif_with_hack, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.dif_with_hack.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TypeTernary {
    pub fn dif(
        &self
    ) -> KResult<Ref<'_, OptRc<TypeTernary_Dummy>>> {
        let _io = self._io.borrow();
        if self.f_dif.get() {
            return Ok(self.dif.borrow());
        }
        *self.dif.borrow_mut() = if !(*self.is_hack()?) { self.dif_wo_hack().clone() } else { self.dif_with_hack().clone() }.clone();
        Ok(self.dif.borrow())
    }
    pub fn dif_value(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_dif_value.get() {
            return Ok(self.dif_value.borrow());
        }
        self.f_dif_value.set(true);
        *self.dif_value.borrow_mut() = (*self.dif()?.value()).try_into()?;
        Ok(self.dif_value.borrow())
    }
    pub fn is_hack(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_hack.get() {
            return Ok(self.is_hack.borrow());
        }
        self.f_is_hack.set(true);
        *self.is_hack.borrow_mut() = (true).try_into()?;
        Ok(self.is_hack.borrow())
    }
}
impl TypeTernary {
    pub fn dif_wo_hack(&self) -> Ref<'_, OptRc<TypeTernary_Dummy>> {
        self.dif_wo_hack.borrow()
    }
}
impl TypeTernary {
    pub fn dif_with_hack(&self) -> Ref<'_, OptRc<TypeTernary_Dummy>> {
        self.dif_with_hack.borrow()
    }
}
impl TypeTernary {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl TypeTernary {
    pub fn dif_wo_hack_raw(&self) -> Ref<'_, Vec<u8>> {
        self.dif_wo_hack_raw.borrow()
    }
}
impl TypeTernary {
    pub fn dif_with_hack_raw(&self) -> Ref<'_, Vec<u8>> {
        self.dif_with_hack_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct TypeTernary_Dummy {
    pub(crate) _root: SharedType<TypeTernary>,
    pub(crate) _parent: SharedType<TypeTernary>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for TypeTernary_Dummy {
    type Root = TypeTernary;
    type Parent = TypeTernary;

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
        *self_rc.value.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TypeTernary_Dummy {
}
impl TypeTernary_Dummy {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl TypeTernary_Dummy {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
