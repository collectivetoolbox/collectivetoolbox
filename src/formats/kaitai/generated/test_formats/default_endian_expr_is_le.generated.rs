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
pub struct DefaultEndianExprIsLe {
    pub(crate) _root: SharedType<DefaultEndianExprIsLe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsLe>,
    pub(crate) _self_shared: SharedType<Self>,
    docs: RefCell<Vec<OptRc<DefaultEndianExprIsLe_Doc>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DefaultEndianExprIsLe {
    type Root = DefaultEndianExprIsLe;
    type Parent = DefaultEndianExprIsLe;

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
        *self_rc.docs.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, DefaultEndianExprIsLe_Doc>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.docs.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprIsLe {
}
impl DefaultEndianExprIsLe {
    pub fn docs(&self) -> Ref<'_, Vec<OptRc<DefaultEndianExprIsLe_Doc>>> {
        self.docs.borrow()
    }
}
impl DefaultEndianExprIsLe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprIsLe_Doc {
    pub(crate) _root: SharedType<DefaultEndianExprIsLe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsLe>,
    pub(crate) _self_shared: SharedType<Self>,
    indicator: RefCell<Vec<u8>>,
    main: RefCell<OptRc<DefaultEndianExprIsLe_Doc_MainObj>>,
    _io: RefCell<BytesReader>,
    indicator_raw: RefCell<Vec<u8>>,
}
impl KStruct for DefaultEndianExprIsLe_Doc {
    type Root = DefaultEndianExprIsLe;
    type Parent = DefaultEndianExprIsLe;

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
        *self_rc.indicator.borrow_mut() = _io.read_bytes(2_usize)?;
        let t = Self::read_into::<_, DefaultEndianExprIsLe_Doc_MainObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprIsLe_Doc {
}
impl DefaultEndianExprIsLe_Doc {
    pub fn indicator(&self) -> Ref<'_, Vec<u8>> {
        self.indicator.borrow()
    }
}
impl DefaultEndianExprIsLe_Doc {
    pub fn main(&self) -> Ref<'_, OptRc<DefaultEndianExprIsLe_Doc_MainObj>> {
        self.main.borrow()
    }
}
impl DefaultEndianExprIsLe_Doc {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DefaultEndianExprIsLe_Doc {
    pub fn indicator_raw(&self) -> Ref<'_, Vec<u8>> {
        self.indicator_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DefaultEndianExprIsLe_Doc_MainObj {
    pub(crate) _root: SharedType<DefaultEndianExprIsLe>,
    pub(crate) _parent: SharedType<DefaultEndianExprIsLe_Doc>,
    pub(crate) _self_shared: SharedType<Self>,
    some_int: RefCell<u32>,
    some_int_be: RefCell<u16>,
    some_int_le: RefCell<u16>,
    _io: RefCell<BytesReader>,
    _is_le: RefCell<i32>,
}
impl KStruct for DefaultEndianExprIsLe_Doc_MainObj {
    type Root = DefaultEndianExprIsLe;
    type Parent = DefaultEndianExprIsLe_Doc;

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
        *self_rc.some_int.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_u4le()? } else { _io.read_u4be()? };
        *self_rc.some_int_be.borrow_mut() = _io.read_u2be()?;
        *self_rc.some_int_le.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DefaultEndianExprIsLe_Doc_MainObj {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl DefaultEndianExprIsLe_Doc_MainObj {
}
impl DefaultEndianExprIsLe_Doc_MainObj {
    pub fn some_int(&self) -> Ref<'_, u32> {
        self.some_int.borrow()
    }
}
impl DefaultEndianExprIsLe_Doc_MainObj {
    pub fn some_int_be(&self) -> Ref<'_, u16> {
        self.some_int_be.borrow()
    }
}
impl DefaultEndianExprIsLe_Doc_MainObj {
    pub fn some_int_le(&self) -> Ref<'_, u16> {
        self.some_int_le.borrow()
    }
}
impl DefaultEndianExprIsLe_Doc_MainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
