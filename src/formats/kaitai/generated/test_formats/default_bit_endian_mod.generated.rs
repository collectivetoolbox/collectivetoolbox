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
pub struct DefaultBitEndianMod {
    pub(crate) _root: SharedType<DefaultBitEndianMod>,
    pub(crate) _parent: SharedType<DefaultBitEndianMod>,
    pub(crate) _self_shared: SharedType<Self>,
    main: RefCell<OptRc<DefaultBitEndianMod_MainObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultBitEndianMod {
    type Root = DefaultBitEndianMod;
    type Parent = DefaultBitEndianMod;

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
        let t = Self::read_into::<_, DefaultBitEndianMod_MainObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultBitEndianMod {
}
impl DefaultBitEndianMod {
    pub fn main(&self) -> Ref<'_, OptRc<DefaultBitEndianMod_MainObj>> {
        self.main.borrow()
    }
}
impl DefaultBitEndianMod {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultBitEndianMod_MainObj {
    pub(crate) _root: SharedType<DefaultBitEndianMod>,
    pub(crate) _parent: SharedType<DefaultBitEndianMod>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u64>,
    two: RefCell<u64>,
    nest: RefCell<OptRc<DefaultBitEndianMod_MainObj_Subnest>>,
    nest_be: RefCell<OptRc<DefaultBitEndianMod_MainObj_SubnestBe>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultBitEndianMod_MainObj {
    type Root = DefaultBitEndianMod;
    type Parent = DefaultBitEndianMod;

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
        *self_rc.one.borrow_mut() = _io.read_bits_int_le(9)?;
        *self_rc.two.borrow_mut() = _io.read_bits_int_le(15)?;
        io.align_to_byte()?;
        let t = Self::read_into::<_, DefaultBitEndianMod_MainObj_Subnest>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.nest.borrow_mut() = t;
        let t = Self::read_into::<_, DefaultBitEndianMod_MainObj_SubnestBe>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.nest_be.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultBitEndianMod_MainObj {
}
impl DefaultBitEndianMod_MainObj {
    pub fn one(&self) -> Ref<'_, u64> {
        self.one.borrow()
    }
}
impl DefaultBitEndianMod_MainObj {
    pub fn two(&self) -> Ref<'_, u64> {
        self.two.borrow()
    }
}
impl DefaultBitEndianMod_MainObj {
    pub fn nest(&self) -> Ref<'_, OptRc<DefaultBitEndianMod_MainObj_Subnest>> {
        self.nest.borrow()
    }
}
impl DefaultBitEndianMod_MainObj {
    pub fn nest_be(&self) -> Ref<'_, OptRc<DefaultBitEndianMod_MainObj_SubnestBe>> {
        self.nest_be.borrow()
    }
}
impl DefaultBitEndianMod_MainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultBitEndianMod_MainObj_Subnest {
    pub(crate) _root: SharedType<DefaultBitEndianMod>,
    pub(crate) _parent: SharedType<DefaultBitEndianMod_MainObj>,
    pub(crate) _self_shared: SharedType<Self>,
    two: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultBitEndianMod_MainObj_Subnest {
    type Root = DefaultBitEndianMod;
    type Parent = DefaultBitEndianMod_MainObj;

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
        *self_rc.two.borrow_mut() = _io.read_bits_int_le(16)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultBitEndianMod_MainObj_Subnest {
}
impl DefaultBitEndianMod_MainObj_Subnest {
    pub fn two(&self) -> Ref<'_, u64> {
        self.two.borrow()
    }
}
impl DefaultBitEndianMod_MainObj_Subnest {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultBitEndianMod_MainObj_SubnestBe {
    pub(crate) _root: SharedType<DefaultBitEndianMod>,
    pub(crate) _parent: SharedType<DefaultBitEndianMod_MainObj>,
    pub(crate) _self_shared: SharedType<Self>,
    two: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultBitEndianMod_MainObj_SubnestBe {
    type Root = DefaultBitEndianMod;
    type Parent = DefaultBitEndianMod_MainObj;

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
        *self_rc.two.borrow_mut() = _io.read_bits_int_be(16)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultBitEndianMod_MainObj_SubnestBe {
}
impl DefaultBitEndianMod_MainObj_SubnestBe {
    pub fn two(&self) -> Ref<'_, u64> {
        self.two.borrow()
    }
}
impl DefaultBitEndianMod_MainObj_SubnestBe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
