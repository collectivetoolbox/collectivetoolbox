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
pub struct EnumIf {
    pub(crate) _root: SharedType<EnumIf>,
    pub(crate) _parent: SharedType<EnumIf>,
    pub(crate) _self_shared: SharedType<Self>,
    op1: RefCell<OptRc<EnumIf_Operation>>,
    op2: RefCell<OptRc<EnumIf_Operation>>,
    op3: RefCell<OptRc<EnumIf_Operation>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumIf {
    type Root = EnumIf;
    type Parent = EnumIf;

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
        let t = Self::read_into::<_, EnumIf_Operation>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.op1.borrow_mut() = t;
        let t = Self::read_into::<_, EnumIf_Operation>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.op2.borrow_mut() = t;
        let t = Self::read_into::<_, EnumIf_Operation>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.op3.borrow_mut() = t;
        Ok(())
    }
}
impl EnumIf {
}
impl EnumIf {
    pub fn op1(&self) -> Ref<'_, OptRc<EnumIf_Operation>> {
        self.op1.borrow()
    }
}
impl EnumIf {
    pub fn op2(&self) -> Ref<'_, OptRc<EnumIf_Operation>> {
        self.op2.borrow()
    }
}
impl EnumIf {
    pub fn op3(&self) -> Ref<'_, OptRc<EnumIf_Operation>> {
        self.op3.borrow()
    }
}
impl EnumIf {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumIf_Opcodes {
    AString,
    ATuple,
    Unknown(i64),
}

impl TryFrom<i64> for EnumIf_Opcodes {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumIf_Opcodes> {
        match flag {
            83 => Ok(EnumIf_Opcodes::AString),
            84 => Ok(EnumIf_Opcodes::ATuple),
            _ => Ok(EnumIf_Opcodes::Unknown(flag)),
        }
    }
}

impl From<&EnumIf_Opcodes> for i64 {
    fn from(v: &EnumIf_Opcodes) -> Self {
        match *v {
            EnumIf_Opcodes::AString => 83,
            EnumIf_Opcodes::ATuple => 84,
            EnumIf_Opcodes::Unknown(v) => v
        }
    }
}

impl Default for EnumIf_Opcodes {
    fn default() -> Self { EnumIf_Opcodes::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct EnumIf_ArgStr {
    pub(crate) _root: SharedType<EnumIf>,
    pub(crate) _parent: SharedType<EnumIf_Operation>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<u8>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumIf_ArgStr {
    type Root = EnumIf;
    type Parent = EnumIf_Operation;

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
        *self_rc.len.borrow_mut() = _io.read_u1()?;
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len()))?, "UTF-8")?;
        Ok(())
    }
}
impl EnumIf_ArgStr {
}
impl EnumIf_ArgStr {
    pub fn len(&self) -> Ref<'_, u8> {
        self.len.borrow()
    }
}
impl EnumIf_ArgStr {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl EnumIf_ArgStr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EnumIf_ArgTuple {
    pub(crate) _root: SharedType<EnumIf>,
    pub(crate) _parent: SharedType<EnumIf_Operation>,
    pub(crate) _self_shared: SharedType<Self>,
    num1: RefCell<u8>,
    num2: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumIf_ArgTuple {
    type Root = EnumIf;
    type Parent = EnumIf_Operation;

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
        *self_rc.num1.borrow_mut() = _io.read_u1()?;
        *self_rc.num2.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl EnumIf_ArgTuple {
}
impl EnumIf_ArgTuple {
    pub fn num1(&self) -> Ref<'_, u8> {
        self.num1.borrow()
    }
}
impl EnumIf_ArgTuple {
    pub fn num2(&self) -> Ref<'_, u8> {
        self.num2.borrow()
    }
}
impl EnumIf_ArgTuple {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EnumIf_Operation {
    pub(crate) _root: SharedType<EnumIf>,
    pub(crate) _parent: SharedType<EnumIf>,
    pub(crate) _self_shared: SharedType<Self>,
    opcode: RefCell<EnumIf_Opcodes>,
    arg_tuple: RefCell<OptRc<EnumIf_ArgTuple>>,
    arg_str: RefCell<OptRc<EnumIf_ArgStr>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumIf_Operation {
    type Root = EnumIf;
    type Parent = EnumIf;

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
        *self_rc.opcode.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        if *self_rc.opcode() == EnumIf_Opcodes::ATuple {
            let t = Self::read_into::<_, EnumIf_ArgTuple>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.arg_tuple.borrow_mut() = t;
        }
        if *self_rc.opcode() == EnumIf_Opcodes::AString {
            let t = Self::read_into::<_, EnumIf_ArgStr>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.arg_str.borrow_mut() = t;
        }
        Ok(())
    }
}
impl EnumIf_Operation {
}
impl EnumIf_Operation {
    pub fn opcode(&self) -> Ref<'_, EnumIf_Opcodes> {
        self.opcode.borrow()
    }
}
impl EnumIf_Operation {
    pub fn arg_tuple(&self) -> Ref<'_, OptRc<EnumIf_ArgTuple>> {
        self.arg_tuple.borrow()
    }
}
impl EnumIf_Operation {
    pub fn arg_str(&self) -> Ref<'_, OptRc<EnumIf_ArgStr>> {
        self.arg_str.borrow()
    }
}
impl EnumIf_Operation {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
