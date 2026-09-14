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
pub struct IfStruct {
    pub(crate) _root: SharedType<IfStruct>,
    pub(crate) _parent: SharedType<IfStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    op1: RefCell<OptRc<IfStruct_Operation>>,
    op2: RefCell<OptRc<IfStruct_Operation>>,
    op3: RefCell<OptRc<IfStruct_Operation>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IfStruct {
    type Root = IfStruct;
    type Parent = IfStruct;

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
        let t = Self::read_into::<_, IfStruct_Operation>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.op1.borrow_mut() = t;
        let t = Self::read_into::<_, IfStruct_Operation>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.op2.borrow_mut() = t;
        let t = Self::read_into::<_, IfStruct_Operation>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.op3.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IfStruct {
}
impl IfStruct {
    pub fn op1(&self) -> Ref<'_, OptRc<IfStruct_Operation>> {
        self.op1.borrow()
    }
}
impl IfStruct {
    pub fn op2(&self) -> Ref<'_, OptRc<IfStruct_Operation>> {
        self.op2.borrow()
    }
}
impl IfStruct {
    pub fn op3(&self) -> Ref<'_, OptRc<IfStruct_Operation>> {
        self.op3.borrow()
    }
}
impl IfStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IfStruct_ArgStr {
    pub(crate) _root: SharedType<IfStruct>,
    pub(crate) _parent: SharedType<IfStruct_Operation>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<u8>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IfStruct_ArgStr {
    type Root = IfStruct;
    type Parent = IfStruct_Operation;

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
        *self_rc.len.borrow_mut() = _io.read_u1()?;
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len()))?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IfStruct_ArgStr {
}
impl IfStruct_ArgStr {
    pub fn len(&self) -> Ref<'_, u8> {
        self.len.borrow()
    }
}
impl IfStruct_ArgStr {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl IfStruct_ArgStr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IfStruct_ArgTuple {
    pub(crate) _root: SharedType<IfStruct>,
    pub(crate) _parent: SharedType<IfStruct_Operation>,
    pub(crate) _self_shared: SharedType<Self>,
    num1: RefCell<u8>,
    num2: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IfStruct_ArgTuple {
    type Root = IfStruct;
    type Parent = IfStruct_Operation;

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
        *self_rc.num1.borrow_mut() = _io.read_u1()?;
        *self_rc.num2.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IfStruct_ArgTuple {
}
impl IfStruct_ArgTuple {
    pub fn num1(&self) -> Ref<'_, u8> {
        self.num1.borrow()
    }
}
impl IfStruct_ArgTuple {
    pub fn num2(&self) -> Ref<'_, u8> {
        self.num2.borrow()
    }
}
impl IfStruct_ArgTuple {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IfStruct_Operation {
    pub(crate) _root: SharedType<IfStruct>,
    pub(crate) _parent: SharedType<IfStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    opcode: RefCell<u8>,
    arg_tuple: RefCell<OptRc<IfStruct_ArgTuple>>,
    arg_str: RefCell<OptRc<IfStruct_ArgStr>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IfStruct_Operation {
    type Root = IfStruct;
    type Parent = IfStruct;

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
        *self_rc.opcode.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.opcode())) == (to_i128(84))) {
            let t = Self::read_into::<_, IfStruct_ArgTuple>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.arg_tuple.borrow_mut() = t;
        }
        if ((to_i128(*self_rc.opcode())) == (to_i128(83))) {
            let t = Self::read_into::<_, IfStruct_ArgStr>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.arg_str.borrow_mut() = t;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IfStruct_Operation {
}
impl IfStruct_Operation {
    pub fn opcode(&self) -> Ref<'_, u8> {
        self.opcode.borrow()
    }
}
impl IfStruct_Operation {
    pub fn arg_tuple(&self) -> Ref<'_, OptRc<IfStruct_ArgTuple>> {
        self.arg_tuple.borrow()
    }
}
impl IfStruct_Operation {
    pub fn arg_str(&self) -> Ref<'_, OptRc<IfStruct_ArgStr>> {
        self.arg_str.borrow()
    }
}
impl IfStruct_Operation {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
