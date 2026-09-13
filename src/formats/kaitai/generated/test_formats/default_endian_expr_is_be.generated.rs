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
pub struct DefaultEndianExprIsBe {
    pub(crate) _root: SharedType<DefaultEndianExprIsBe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsBe>,
    pub(crate) _self_shared: SharedType<Self>,
    docs: RefCell<Vec<OptRc<DefaultEndianExprIsBe_Doc>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultEndianExprIsBe {
    type Root = DefaultEndianExprIsBe;
    type Parent = DefaultEndianExprIsBe;

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
        *self_rc.docs.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, DefaultEndianExprIsBe_Doc>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.docs.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl DefaultEndianExprIsBe {
}
impl DefaultEndianExprIsBe {
    pub fn docs(&self) -> Ref<'_, Vec<OptRc<DefaultEndianExprIsBe_Doc>>> {
        self.docs.borrow()
    }
}
impl DefaultEndianExprIsBe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprIsBe_Doc {
    pub(crate) _root: SharedType<DefaultEndianExprIsBe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsBe>,
    pub(crate) _self_shared: SharedType<Self>,
    indicator: RefCell<Vec<u8>>,
    main: RefCell<OptRc<DefaultEndianExprIsBe_Doc_MainObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultEndianExprIsBe_Doc {
    type Root = DefaultEndianExprIsBe;
    type Parent = DefaultEndianExprIsBe;

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
        *self_rc.indicator.borrow_mut() = _io.read_bytes(2_usize)?;
        let t = Self::read_into::<_, DefaultEndianExprIsBe_Doc_MainObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        Ok(())
    }
}
impl DefaultEndianExprIsBe_Doc {
}
impl DefaultEndianExprIsBe_Doc {
    pub fn indicator(&self) -> Ref<'_, Vec<u8>> {
        self.indicator.borrow()
    }
}
impl DefaultEndianExprIsBe_Doc {
    pub fn main(&self) -> Ref<'_, OptRc<DefaultEndianExprIsBe_Doc_MainObj>> {
        self.main.borrow()
    }
}
impl DefaultEndianExprIsBe_Doc {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprIsBe_Doc_MainObj {
    pub(crate) _root: SharedType<DefaultEndianExprIsBe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsBe_Doc>,
    pub(crate) _self_shared: SharedType<Self>,
    some_int: RefCell<u32>,
    some_int_be: RefCell<u16>,
    some_int_le: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_inst_int: Cell<bool>,
    inst_int: RefCell<u32>,
    f_inst_sub: Cell<bool>,
    inst_sub: RefCell<OptRc<DefaultEndianExprIsBe_Doc_MainObj_SubMainObj>>,
    _is_le: RefCell<i32>,
}
impl KStruct for DefaultEndianExprIsBe_Doc_MainObj {
    type Root = DefaultEndianExprIsBe;
    type Parent = DefaultEndianExprIsBe_Doc;

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
        match *self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.indicator() {
            [0x4d, 0x4d] => {
                *self_rc._is_le.borrow_mut() = 2_i32;
            }
            _ => {
                *self_rc._is_le.borrow_mut() = 1_i32;
            }
            _ => {}
        }
        if *self_rc._is_le.borrow() == 0 {
            return Err(KError::UndecidedEndianness { src_path: "/types/doc/types/main_obj".to_string() });
        }
        *self_rc.some_int.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_u4le()? } else { _io.read_u4be()? };
        *self_rc.some_int_be.borrow_mut() = _io.read_u2be()?;
        *self_rc.some_int_le.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj {
    pub fn inst_int(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_inst_int.get() {
            return Ok(self.inst_int.borrow());
        }
        self.f_inst_int.set(true);
        let _pos = _io.pos();
        _io.seek(2_usize)?;
        *self.inst_int.borrow_mut() = if *self._is_le.borrow() == 1 { _io.read_u4le()? } else { _io.read_u4be()? };
        _io.seek(_pos)?;
        Ok(self.inst_int.borrow())
    }
    pub fn inst_sub(
        &self
    ) -> KResult<Ref<'_, OptRc<DefaultEndianExprIsBe_Doc_MainObj_SubMainObj>>> {
        let _io = self._io.borrow();
        if self.f_inst_sub.get() {
            return Ok(self.inst_sub.borrow());
        }
        let _pos = _io.pos();
        _io.seek(2_usize)?;
        let f = |t : &mut DefaultEndianExprIsBe_Doc_MainObj_SubMainObj| Ok(t.set_endian(*self._is_le.borrow()));
        let t = Self::read_into_with_init::<_, DefaultEndianExprIsBe_Doc_MainObj_SubMainObj>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()), &f)?.into();
        *self.inst_sub.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.inst_sub.borrow())
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj {
    pub fn some_int(&self) -> Ref<'_, u32> {
        self.some_int.borrow()
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj {
    pub fn some_int_be(&self) -> Ref<'_, u16> {
        self.some_int_be.borrow()
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj {
    pub fn some_int_le(&self) -> Ref<'_, u16> {
        self.some_int_le.borrow()
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprIsBe_Doc_MainObj_SubMainObj {
    pub(crate) _root: SharedType<DefaultEndianExprIsBe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsBe_Doc_MainObj>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u32>,
    _io: RefCell<BytesReader>,
    _is_le: RefCell<i32>,
}
impl KStruct for DefaultEndianExprIsBe_Doc_MainObj_SubMainObj {
    type Root = DefaultEndianExprIsBe;
    type Parent = DefaultEndianExprIsBe_Doc_MainObj;

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
        *self_rc.foo.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_u4le()? } else { _io.read_u4be()? };
        Ok(())
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj_SubMainObj {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj_SubMainObj {
}
impl DefaultEndianExprIsBe_Doc_MainObj_SubMainObj {
    pub fn foo(&self) -> Ref<'_, u32> {
        self.foo.borrow()
    }
}
impl DefaultEndianExprIsBe_Doc_MainObj_SubMainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
