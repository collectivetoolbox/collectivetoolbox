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
pub struct DefaultEndianExprInherited {
    pub(crate) _root: SharedType<DefaultEndianExprInherited>,
    pub(crate) _parent: SharedType<DefaultEndianExprInherited>,
    pub(crate) _self_shared: SharedType<Self>,
    docs: RefCell<Vec<OptRc<DefaultEndianExprInherited_Doc>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultEndianExprInherited {
    type Root = DefaultEndianExprInherited;
    type Parent = DefaultEndianExprInherited;

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
                let t = Self::read_into::<_, DefaultEndianExprInherited_Doc>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.docs.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprInherited {
}
impl DefaultEndianExprInherited {
    pub fn docs(&self) -> Ref<'_, Vec<OptRc<DefaultEndianExprInherited_Doc>>> {
        self.docs.borrow()
    }
}
impl DefaultEndianExprInherited {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprInherited_Doc {
    pub(crate) _root: SharedType<DefaultEndianExprInherited>,
    pub(crate) _parent: SharedType<DefaultEndianExprInherited>,
    pub(crate) _self_shared: SharedType<Self>,
    indicator: RefCell<Vec<u8>>,
    main: RefCell<OptRc<DefaultEndianExprInherited_Doc_MainObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultEndianExprInherited_Doc {
    type Root = DefaultEndianExprInherited;
    type Parent = DefaultEndianExprInherited;

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
        let t = Self::read_into::<_, DefaultEndianExprInherited_Doc_MainObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprInherited_Doc {
}
impl DefaultEndianExprInherited_Doc {
    pub fn indicator(&self) -> Ref<'_, Vec<u8>> {
        self.indicator.borrow()
    }
}
impl DefaultEndianExprInherited_Doc {
    pub fn main(&self) -> Ref<'_, OptRc<DefaultEndianExprInherited_Doc_MainObj>> {
        self.main.borrow()
    }
}
impl DefaultEndianExprInherited_Doc {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprInherited_Doc_MainObj {
    pub(crate) _root: SharedType<DefaultEndianExprInherited>,
    pub(crate) _parent: SharedType<DefaultEndianExprInherited_Doc>,
    pub(crate) _self_shared: SharedType<Self>,
    insides: RefCell<OptRc<DefaultEndianExprInherited_Doc_MainObj_SubObj>>,
    _io: RefCell<BytesReader>,
    _is_le: RefCell<i32>,
}
impl KStruct for DefaultEndianExprInherited_Doc_MainObj {
    type Root = DefaultEndianExprInherited;
    type Parent = DefaultEndianExprInherited_Doc;

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
            [0x49, 0x49] => {
                *self_rc._is_le.borrow_mut() = 1_i32;
            }
            _ => {
                *self_rc._is_le.borrow_mut() = 2_i32;
            }
            _ => {}
        }
        if *self_rc._is_le.borrow() == 0 {
            return Err(KError::UndecidedEndianness { src_path: "/types/doc/types/main_obj".to_string() });
        }
        let f = |t : &mut DefaultEndianExprInherited_Doc_MainObj_SubObj| Ok(t.set_endian(*self_rc._is_le.borrow()));
        let t = Self::read_into_with_init::<_, DefaultEndianExprInherited_Doc_MainObj_SubObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.insides.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprInherited_Doc_MainObj {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl DefaultEndianExprInherited_Doc_MainObj {
}
impl DefaultEndianExprInherited_Doc_MainObj {
    pub fn insides(&self) -> Ref<'_, OptRc<DefaultEndianExprInherited_Doc_MainObj_SubObj>> {
        self.insides.borrow()
    }
}
impl DefaultEndianExprInherited_Doc_MainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprInherited_Doc_MainObj_SubObj {
    pub(crate) _root: SharedType<DefaultEndianExprInherited>,
    pub(crate) _parent: SharedType<DefaultEndianExprInherited_Doc_MainObj>,
    pub(crate) _self_shared: SharedType<Self>,
    some_int: RefCell<u32>,
    more: RefCell<OptRc<DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj>>,
    _io: RefCell<BytesReader>,
    _is_le: RefCell<i32>,
}
impl KStruct for DefaultEndianExprInherited_Doc_MainObj_SubObj {
    type Root = DefaultEndianExprInherited;
    type Parent = DefaultEndianExprInherited_Doc_MainObj;

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
        *self_rc.some_int.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_u4le()? } else { _io.read_u4be()? };
        let f = |t : &mut DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj| Ok(t.set_endian(*self_rc._is_le.borrow()));
        let t = Self::read_into_with_init::<_, DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.more.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj {
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj {
    pub fn some_int(&self) -> Ref<'_, u32> {
        self.some_int.borrow()
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj {
    pub fn more(&self) -> Ref<'_, OptRc<DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj>> {
        self.more.borrow()
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    pub(crate) _root: SharedType<DefaultEndianExprInherited>,
    pub(crate) _parent: SharedType<DefaultEndianExprInherited_Doc_MainObj_SubObj>,
    pub(crate) _self_shared: SharedType<Self>,
    some_int1: RefCell<u16>,
    some_int2: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_some_inst: Cell<bool>,
    some_inst: RefCell<u32>,
    _is_le: RefCell<i32>,
}
impl KStruct for DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    type Root = DefaultEndianExprInherited;
    type Parent = DefaultEndianExprInherited_Doc_MainObj_SubObj;

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
        *self_rc.some_int1.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_u2le()? } else { _io.read_u2be()? };
        *self_rc.some_int2.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_u2le()? } else { _io.read_u2be()? };
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    pub fn some_inst(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_some_inst.get() {
            return Ok(self.some_inst.borrow());
        }
        self.f_some_inst.set(true);
        let _pos = _io.pos();
        _io.seek(2_usize)?;
        *self.some_inst.borrow_mut() = if *self._is_le.borrow() == 1 { _io.read_u4le()? } else { _io.read_u4be()? };
        _io.seek(_pos)?;
        Ok(self.some_inst.borrow())
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    pub fn some_int1(&self) -> Ref<'_, u16> {
        self.some_int1.borrow()
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    pub fn some_int2(&self) -> Ref<'_, u16> {
        self.some_int2.borrow()
    }
}
impl DefaultEndianExprInherited_Doc_MainObj_SubObj_SubsubObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
