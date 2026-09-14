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
pub struct RecursiveOne {
    pub(crate) _root: SharedType<RecursiveOne>,
    pub(crate) _parent: SharedType<RecursiveOne>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u8>,
    next: RefCell<Option<RecursiveOne_Next>>,
    _io: RefCell<BytesReader>,
    next_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum RecursiveOne_Next {
    RecursiveOne(OptRc<RecursiveOne>),
    RecursiveOne_Fini(OptRc<RecursiveOne_Fini>),
}
impl TryFrom<&RecursiveOne_Next> for OptRc<RecursiveOne> {
    type Error = KError;
    fn try_from(v: &RecursiveOne_Next) -> Result<Self, Self::Error> {
        if let RecursiveOne_Next::RecursiveOne(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RecursiveOne>> for RecursiveOne_Next {
    fn from(v: OptRc<RecursiveOne>) -> Self {
        Self::RecursiveOne(v)
    }
}
impl TryFrom<&RecursiveOne_Next> for OptRc<RecursiveOne_Fini> {
    type Error = KError;
    fn try_from(v: &RecursiveOne_Next) -> Result<Self, Self::Error> {
        if let RecursiveOne_Next::RecursiveOne_Fini(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RecursiveOne_Fini>> for RecursiveOne_Next {
    fn from(v: OptRc<RecursiveOne_Fini>) -> Self {
        Self::RecursiveOne_Fini(v)
    }
}
impl KStruct for RecursiveOne {
    type Root = RecursiveOne;
    type Parent = RecursiveOne;

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
        *self_rc.one.borrow_mut() = _io.read_u1()?;
        match ((i32::from(*self_rc.one())) & (3_i32)) {
            0 => {
                *self_rc.next_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let next_raw = self_rc.next_raw.borrow();
                let _t_next_raw_io = BytesReader::from(next_raw.clone());
                let t = Self::read_into::<BytesReader, RecursiveOne>(&_t_next_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.next.borrow_mut() = Some(t);
            }
            1 => {
                *self_rc.next_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let next_raw = self_rc.next_raw.borrow();
                let _t_next_raw_io = BytesReader::from(next_raw.clone());
                let t = Self::read_into::<BytesReader, RecursiveOne>(&_t_next_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.next.borrow_mut() = Some(t);
            }
            2 => {
                *self_rc.next_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let next_raw = self_rc.next_raw.borrow();
                let _t_next_raw_io = BytesReader::from(next_raw.clone());
                let t = Self::read_into::<BytesReader, RecursiveOne>(&_t_next_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.next.borrow_mut() = Some(t);
            }
            3 => {
                *self_rc.next_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let next_raw = self_rc.next_raw.borrow();
                let _t_next_raw_io = BytesReader::from(next_raw.clone());
                let t = Self::read_into::<BytesReader, RecursiveOne_Fini>(&_t_next_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.next.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RecursiveOne {
}
impl RecursiveOne {
    pub fn one(&self) -> Ref<'_, u8> {
        self.one.borrow()
    }
}
impl RecursiveOne {
    pub fn next(&self) -> Ref<'_, Option<RecursiveOne_Next>> {
        self.next.borrow()
    }
}
impl RecursiveOne {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl RecursiveOne {
    pub fn next_raw(&self) -> Ref<'_, Vec<u8>> {
        self.next_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RecursiveOne_Fini {
    pub(crate) _root: SharedType<RecursiveOne>,
    pub(crate) _parent: SharedType<RecursiveOne>,
    pub(crate) _self_shared: SharedType<Self>,
    finisher: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RecursiveOne_Fini {
    type Root = RecursiveOne;
    type Parent = RecursiveOne;

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
        *self_rc.finisher.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RecursiveOne_Fini {
}
impl RecursiveOne_Fini {
    pub fn finisher(&self) -> Ref<'_, u16> {
        self.finisher.borrow()
    }
}
impl RecursiveOne_Fini {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
