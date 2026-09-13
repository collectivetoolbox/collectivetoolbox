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
pub struct DebugEnumName {
    pub(crate) _root: SharedType<DebugEnumName>,
    pub(crate) _parent: SharedType<DebugEnumName>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<DebugEnumName_TestEnum1>,
    array_of_ints: RefCell<Vec<DebugEnumName_TestEnum2>>,
    test_type: RefCell<OptRc<DebugEnumName_TestSubtype>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DebugEnumName {
    type Root = DebugEnumName;
    type Parent = DebugEnumName;

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
        *self_rc.one.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.array_of_ints.borrow_mut() = Vec::new();
        let l_array_of_ints = 1_usize;
        for _i in 0_usize..l_array_of_ints {
            self_rc.array_of_ints.borrow_mut().push(i64::from(_io.read_u1()?).try_into()?);
        }
        let t = Self::read_into::<_, DebugEnumName_TestSubtype>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.test_type.borrow_mut() = t;
        Ok(())
    }
}
impl DebugEnumName {
}
impl DebugEnumName {
    pub fn one(&self) -> Ref<'_, DebugEnumName_TestEnum1> {
        self.one.borrow()
    }
}
impl DebugEnumName {
    pub fn array_of_ints(&self) -> Ref<'_, Vec<DebugEnumName_TestEnum2>> {
        self.array_of_ints.borrow()
    }
}
impl DebugEnumName {
    pub fn test_type(&self) -> Ref<'_, OptRc<DebugEnumName_TestSubtype>> {
        self.test_type.borrow()
    }
}
impl DebugEnumName {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DebugEnumName_TestEnum1 {
    EnumValue80,
    Unknown(i64),
}

impl TryFrom<i64> for DebugEnumName_TestEnum1 {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<DebugEnumName_TestEnum1> {
        match flag {
            80 => Ok(DebugEnumName_TestEnum1::EnumValue80),
            _ => Ok(DebugEnumName_TestEnum1::Unknown(flag)),
        }
    }
}

impl From<&DebugEnumName_TestEnum1> for i64 {
    fn from(v: &DebugEnumName_TestEnum1) -> Self {
        match *v {
            DebugEnumName_TestEnum1::EnumValue80 => 80,
            DebugEnumName_TestEnum1::Unknown(v) => v
        }
    }
}

impl Default for DebugEnumName_TestEnum1 {
    fn default() -> Self { DebugEnumName_TestEnum1::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DebugEnumName_TestEnum2 {
    EnumValue65,
    Unknown(i64),
}

impl TryFrom<i64> for DebugEnumName_TestEnum2 {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<DebugEnumName_TestEnum2> {
        match flag {
            65 => Ok(DebugEnumName_TestEnum2::EnumValue65),
            _ => Ok(DebugEnumName_TestEnum2::Unknown(flag)),
        }
    }
}

impl From<&DebugEnumName_TestEnum2> for i64 {
    fn from(v: &DebugEnumName_TestEnum2) -> Self {
        match *v {
            DebugEnumName_TestEnum2::EnumValue65 => 65,
            DebugEnumName_TestEnum2::Unknown(v) => v
        }
    }
}

impl Default for DebugEnumName_TestEnum2 {
    fn default() -> Self { DebugEnumName_TestEnum2::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct DebugEnumName_TestSubtype {
    pub(crate) _root: SharedType<DebugEnumName>,
    pub(crate) _parent: SharedType<DebugEnumName>,
    pub(crate) _self_shared: SharedType<Self>,
    field1: RefCell<DebugEnumName_TestSubtype_InnerEnum1>,
    field2: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_instance_field: Cell<bool>,
    instance_field: RefCell<DebugEnumName_TestSubtype_InnerEnum2>,
}
impl KStruct for DebugEnumName_TestSubtype {
    type Root = DebugEnumName;
    type Parent = DebugEnumName;

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
        *self_rc.field1.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.field2.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl DebugEnumName_TestSubtype {
    pub fn instance_field(
        &self
    ) -> KResult<Ref<'_, DebugEnumName_TestSubtype_InnerEnum2>> {
        let _io = self._io.borrow();
        if self.f_instance_field.get() {
            return Ok(self.instance_field.borrow());
        }
        self.f_instance_field.set(true);
        *self.instance_field.borrow_mut() = i64::from(((i32::from(*self.field2())) & (15_i32))).try_into()?;
        Ok(self.instance_field.borrow())
    }
}
impl DebugEnumName_TestSubtype {
    pub fn field1(&self) -> Ref<'_, DebugEnumName_TestSubtype_InnerEnum1> {
        self.field1.borrow()
    }
}
impl DebugEnumName_TestSubtype {
    pub fn field2(&self) -> Ref<'_, u8> {
        self.field2.borrow()
    }
}
impl DebugEnumName_TestSubtype {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DebugEnumName_TestSubtype_InnerEnum1 {
    EnumValue67,
    Unknown(i64),
}

impl TryFrom<i64> for DebugEnumName_TestSubtype_InnerEnum1 {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<DebugEnumName_TestSubtype_InnerEnum1> {
        match flag {
            67 => Ok(DebugEnumName_TestSubtype_InnerEnum1::EnumValue67),
            _ => Ok(DebugEnumName_TestSubtype_InnerEnum1::Unknown(flag)),
        }
    }
}

impl From<&DebugEnumName_TestSubtype_InnerEnum1> for i64 {
    fn from(v: &DebugEnumName_TestSubtype_InnerEnum1) -> Self {
        match *v {
            DebugEnumName_TestSubtype_InnerEnum1::EnumValue67 => 67,
            DebugEnumName_TestSubtype_InnerEnum1::Unknown(v) => v
        }
    }
}

impl Default for DebugEnumName_TestSubtype_InnerEnum1 {
    fn default() -> Self { DebugEnumName_TestSubtype_InnerEnum1::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DebugEnumName_TestSubtype_InnerEnum2 {
    EnumValue11,
    Unknown(i64),
}

impl TryFrom<i64> for DebugEnumName_TestSubtype_InnerEnum2 {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<DebugEnumName_TestSubtype_InnerEnum2> {
        match flag {
            11 => Ok(DebugEnumName_TestSubtype_InnerEnum2::EnumValue11),
            _ => Ok(DebugEnumName_TestSubtype_InnerEnum2::Unknown(flag)),
        }
    }
}

impl From<&DebugEnumName_TestSubtype_InnerEnum2> for i64 {
    fn from(v: &DebugEnumName_TestSubtype_InnerEnum2) -> Self {
        match *v {
            DebugEnumName_TestSubtype_InnerEnum2::EnumValue11 => 11,
            DebugEnumName_TestSubtype_InnerEnum2::Unknown(v) => v
        }
    }
}

impl Default for DebugEnumName_TestSubtype_InnerEnum2 {
    fn default() -> Self { DebugEnumName_TestSubtype_InnerEnum2::Unknown(0) }
}

