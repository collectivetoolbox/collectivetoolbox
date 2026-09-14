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
pub struct ParamsPassArrayStr {
    pub(crate) _root: SharedType<ParamsPassArrayStr>,
    pub(crate) _parent: SharedType<ParamsPassArrayStr>,
    pub(crate) _self_shared: SharedType<Self>,
    str_array: RefCell<Vec<String>>,
    pass_str_array: RefCell<OptRc<ParamsPassArrayStr_WantsStrs>>,
    pass_str_array_calc: RefCell<OptRc<ParamsPassArrayStr_WantsStrs>>,
    _io: RefCell<BytesReader>,
    f_str_array_calc: Cell<bool>,
    str_array_calc: RefCell<Vec<String>>,
}
impl KStruct for ParamsPassArrayStr {
    type Root = ParamsPassArrayStr;
    type Parent = ParamsPassArrayStr;

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
        *self_rc.str_array.borrow_mut() = Vec::new();
        let l_str_array = 3_usize;
        for _i in 0_usize..l_str_array {
            self_rc.str_array.borrow_mut().push(bytes_to_str(&_io.read_bytes(2_usize)?, "ascii")?);
        }
        let f = |t : &mut ParamsPassArrayStr_WantsStrs| Ok(t.set_params(self_rc.str_array().clone()));
        let t = Self::read_into_with_init::<_, ParamsPassArrayStr_WantsStrs>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.pass_str_array.borrow_mut() = t;
        let f = |t : &mut ParamsPassArrayStr_WantsStrs| Ok(t.set_params(*self_rc.str_array_calc()?.clone()));
        let t = Self::read_into_with_init::<_, ParamsPassArrayStr_WantsStrs>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.pass_str_array_calc.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassArrayStr {
    pub fn str_array_calc(
        &self
    ) -> KResult<Ref<'_, Vec<String>>> {
        let _io = self._io.borrow();
        if self.f_str_array_calc.get() {
            return Ok(self.str_array_calc.borrow());
        }
        self.f_str_array_calc.set(true);
        *self.str_array_calc.borrow_mut() = vec!["aB".to_string(), "Cd".to_string()];
        Ok(self.str_array_calc.borrow())
    }
}
impl ParamsPassArrayStr {
    pub fn str_array(&self) -> Ref<'_, Vec<String>> {
        self.str_array.borrow()
    }
}
impl ParamsPassArrayStr {
    pub fn pass_str_array(&self) -> Ref<'_, OptRc<ParamsPassArrayStr_WantsStrs>> {
        self.pass_str_array.borrow()
    }
}
impl ParamsPassArrayStr {
    pub fn pass_str_array_calc(&self) -> Ref<'_, OptRc<ParamsPassArrayStr_WantsStrs>> {
        self.pass_str_array_calc.borrow()
    }
}
impl ParamsPassArrayStr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassArrayStr_WantsStrs {
    pub(crate) _root: SharedType<ParamsPassArrayStr>,
    pub(crate) _parent: SharedType<ParamsPassArrayStr>,
    pub(crate) _self_shared: SharedType<Self>,
    strs: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassArrayStr_WantsStrs {
    type Root = ParamsPassArrayStr;
    type Parent = ParamsPassArrayStr;

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
impl ParamsPassArrayStr_WantsStrs {
    pub fn strs(&self) -> Ref<'_, String> {
        self.strs.borrow()
    }
}
impl ParamsPassArrayStr_WantsStrs {
    pub fn set_params(&mut self, strs: String) {
        *self.strs.borrow_mut() = strs;
    }
}
impl ParamsPassArrayStr_WantsStrs {
}
impl ParamsPassArrayStr_WantsStrs {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
