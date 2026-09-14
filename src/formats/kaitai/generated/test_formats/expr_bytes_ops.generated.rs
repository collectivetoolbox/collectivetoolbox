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
pub struct ExprBytesOps {
    pub(crate) _root: SharedType<ExprBytesOps>,
    pub(crate) _parent: SharedType<ExprBytesOps>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_one_first: Cell<bool>,
    one_first: RefCell<i32>,
    f_one_last: Cell<bool>,
    one_last: RefCell<i32>,
    f_one_max: Cell<bool>,
    one_max: RefCell<i32>,
    f_one_mid: Cell<bool>,
    one_mid: RefCell<i32>,
    f_one_min: Cell<bool>,
    one_min: RefCell<i32>,
    f_one_size: Cell<bool>,
    one_size: RefCell<i32>,
    f_two: Cell<bool>,
    two: RefCell<Vec<i32>>,
    f_two_first: Cell<bool>,
    two_first: RefCell<i32>,
    f_two_last: Cell<bool>,
    two_last: RefCell<i32>,
    f_two_max: Cell<bool>,
    two_max: RefCell<i32>,
    f_two_mid: Cell<bool>,
    two_mid: RefCell<i32>,
    f_two_min: Cell<bool>,
    two_min: RefCell<i32>,
    f_two_size: Cell<bool>,
    two_size: RefCell<i32>,
}
impl KStruct for ExprBytesOps {
    type Root = ExprBytesOps;
    type Parent = ExprBytesOps;

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
        *self_rc.one.borrow_mut() = _io.read_bytes(3_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprBytesOps {
    pub fn one_first(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_first.get() {
            return Ok(self.one_first.borrow());
        }
        self.f_one_first.set(true);
        *self.one_first.borrow_mut() = (*self.one().first().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.one_first.borrow())
    }
    pub fn one_last(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_last.get() {
            return Ok(self.one_last.borrow());
        }
        self.f_one_last.set(true);
        *self.one_last.borrow_mut() = (*self.one().last().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.one_last.borrow())
    }
    pub fn one_max(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_max.get() {
            return Ok(self.one_max.borrow());
        }
        self.f_one_max.set(true);
        *self.one_max.borrow_mut() = (*self.one().iter().max().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.one_max.borrow())
    }
    pub fn one_mid(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_mid.get() {
            return Ok(self.one_mid.borrow());
        }
        self.f_one_mid.set(true);
        *self.one_mid.borrow_mut() = (*(self.one().get(1_usize).ok_or(KError::CastError)?)).try_into()?;
        Ok(self.one_mid.borrow())
    }
    pub fn one_min(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_min.get() {
            return Ok(self.one_min.borrow());
        }
        self.f_one_min.set(true);
        *self.one_min.borrow_mut() = (*self.one().iter().min().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.one_min.borrow())
    }
    pub fn one_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_size.get() {
            return Ok(self.one_size.borrow());
        }
        self.f_one_size.set(true);
        *self.one_size.borrow_mut() = (self.one().len()).try_into()?;
        Ok(self.one_size.borrow())
    }
    pub fn two(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_two.get() {
            return Ok(self.two.borrow());
        }
        self.f_two.set(true);
        *self.two.borrow_mut() = vec![65_i32, 255_i32, 75_i32];
        Ok(self.two.borrow())
    }
    pub fn two_first(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_first.get() {
            return Ok(self.two_first.borrow());
        }
        self.f_two_first.set(true);
        *self.two_first.borrow_mut() = (*self.two()?.first().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.two_first.borrow())
    }
    pub fn two_last(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_last.get() {
            return Ok(self.two_last.borrow());
        }
        self.f_two_last.set(true);
        *self.two_last.borrow_mut() = (*self.two()?.last().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.two_last.borrow())
    }
    pub fn two_max(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_max.get() {
            return Ok(self.two_max.borrow());
        }
        self.f_two_max.set(true);
        *self.two_max.borrow_mut() = (*self.two()?.iter().max().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.two_max.borrow())
    }
    pub fn two_mid(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_mid.get() {
            return Ok(self.two_mid.borrow());
        }
        self.f_two_mid.set(true);
        *self.two_mid.borrow_mut() = (*(self.two()?.get(1_usize).ok_or(KError::CastError)?)).try_into()?;
        Ok(self.two_mid.borrow())
    }
    pub fn two_min(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_min.get() {
            return Ok(self.two_min.borrow());
        }
        self.f_two_min.set(true);
        *self.two_min.borrow_mut() = (*self.two()?.iter().min().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.two_min.borrow())
    }
    pub fn two_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_size.get() {
            return Ok(self.two_size.borrow());
        }
        self.f_two_size.set(true);
        *self.two_size.borrow_mut() = (self.two()?.len()).try_into()?;
        Ok(self.two_size.borrow())
    }
}
impl ExprBytesOps {
    pub fn one(&self) -> Ref<'_, Vec<u8>> {
        self.one.borrow()
    }
}
impl ExprBytesOps {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
