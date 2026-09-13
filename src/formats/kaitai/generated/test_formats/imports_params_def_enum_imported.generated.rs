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
use super::params_def_enum_imported::ParamsDefEnumImported;
use super::enum_import_seq::EnumImportSeq;

#[derive(Default, Debug, Clone)]
pub struct ImportsParamsDefEnumImported {
    pub(crate) _root: SharedType<ImportsParamsDefEnumImported>,
    pub(crate) _parent: SharedType<ImportsParamsDefEnumImported>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<OptRc<EnumImportSeq>>,
    two: RefCell<OptRc<ParamsDefEnumImported>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ImportsParamsDefEnumImported {
    type Root = ImportsParamsDefEnumImported;
    type Parent = ImportsParamsDefEnumImported;

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
        let t = Self::read_into::<_, EnumImportSeq>(&*_io, None, None)?.into();
        *self_rc.one.borrow_mut() = t;
        let f = |t : &mut ParamsDefEnumImported| Ok(t.set_params(*self_rc.one().pet_1(), *self_rc.one().pet_2()));
        let t = Self::read_into_with_init::<_, ParamsDefEnumImported>(&*_io, None, None, &f)?.into();
        *self_rc.two.borrow_mut() = t;
        Ok(())
    }
}
impl ImportsParamsDefEnumImported {
}
impl ImportsParamsDefEnumImported {
    pub fn one(&self) -> Ref<'_, OptRc<EnumImportSeq>> {
        self.one.borrow()
    }
}
impl ImportsParamsDefEnumImported {
    pub fn two(&self) -> Ref<'_, OptRc<ParamsDefEnumImported>> {
        self.two.borrow()
    }
}
impl ImportsParamsDefEnumImported {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
