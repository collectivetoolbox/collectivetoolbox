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
pub struct DebugArrayUserEofException {
    pub(crate) _root: SharedType<DebugArrayUserEofException>,
    pub(crate) _parent: SharedType<DebugArrayUserEofException>,
    pub(crate) _self_shared: SharedType<Self>,
    one_cat: RefCell<OptRc<DebugArrayUserEofException_Cat>>,
    array_of_cats: RefCell<Vec<OptRc<DebugArrayUserEofException_Cat>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DebugArrayUserEofException {
    type Root = DebugArrayUserEofException;
    type Parent = DebugArrayUserEofException;

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
        let t = Self::read_into::<_, DebugArrayUserEofException_Cat>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.one_cat.borrow_mut() = t;
        *self_rc.array_of_cats.borrow_mut() = Vec::new();
        let l_array_of_cats = 3_usize;
        for _i in 0_usize..l_array_of_cats {
            let t = Self::read_into::<_, DebugArrayUserEofException_Cat>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.array_of_cats.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugArrayUserEofException {
}
impl DebugArrayUserEofException {
    pub fn one_cat(&self) -> Ref<'_, OptRc<DebugArrayUserEofException_Cat>> {
        self.one_cat.borrow()
    }
}
impl DebugArrayUserEofException {
    pub fn array_of_cats(&self) -> Ref<'_, Vec<OptRc<DebugArrayUserEofException_Cat>>> {
        self.array_of_cats.borrow()
    }
}
impl DebugArrayUserEofException {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DebugArrayUserEofException_Cat {
    pub(crate) _root: SharedType<DebugArrayUserEofException>,
    pub(crate) _parent: SharedType<DebugArrayUserEofException>,
    pub(crate) _self_shared: SharedType<Self>,
    meow: RefCell<u8>,
    chirp: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DebugArrayUserEofException_Cat {
    type Root = DebugArrayUserEofException;
    type Parent = DebugArrayUserEofException;

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
        *self_rc.meow.borrow_mut() = _io.read_u1()?;
        *self_rc.chirp.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugArrayUserEofException_Cat {
}
impl DebugArrayUserEofException_Cat {
    pub fn meow(&self) -> Ref<'_, u8> {
        self.meow.borrow()
    }
}
impl DebugArrayUserEofException_Cat {
    pub fn chirp(&self) -> Ref<'_, u8> {
        self.chirp.borrow()
    }
}
impl DebugArrayUserEofException_Cat {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
