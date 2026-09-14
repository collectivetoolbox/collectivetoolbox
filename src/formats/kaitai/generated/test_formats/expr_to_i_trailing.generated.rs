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
pub struct ExprToITrailing {
    pub(crate) _root: SharedType<ExprToITrailing>,
    pub(crate) _parent: SharedType<ExprToITrailing>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_to_i_garbage: Cell<bool>,
    to_i_garbage: RefCell<i32>,
    f_to_i_r10: Cell<bool>,
    to_i_r10: RefCell<i32>,
    f_to_i_r16: Cell<bool>,
    to_i_r16: RefCell<i32>,
}
impl KStruct for ExprToITrailing {
    type Root = ExprToITrailing;
    type Parent = ExprToITrailing;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprToITrailing {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn to_i_garbage(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_garbage.get() {
            return Ok(self.to_i_garbage.borrow());
        }
        self.f_to_i_garbage.set(true);
        *self.to_i_garbage.borrow_mut() = ("123_.^".parse::<i32>().map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_garbage.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn to_i_r10(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_r10.get() {
            return Ok(self.to_i_r10.borrow());
        }
        self.f_to_i_r10.set(true);
        *self.to_i_r10.borrow_mut() = ("9173abc".parse::<i32>().map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_r10.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn to_i_r16(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_r16.get() {
            return Ok(self.to_i_r16.borrow());
        }
        self.f_to_i_r16.set(true);
        *self.to_i_r16.borrow_mut() = (i32::from_str_radix("9173abc", 16).map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_r16.borrow())
    }
}
impl ExprToITrailing {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
