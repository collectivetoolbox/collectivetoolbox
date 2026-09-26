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
pub struct DebugArrayUserCurrentExcluded {
    pub(crate) _root: SharedType<DebugArrayUserCurrentExcluded>,
    pub(crate) _parent: SharedType<DebugArrayUserCurrentExcluded>,
    pub(crate) _self_shared: SharedType<Self>,
    array_of_cats: RefCell<Vec<OptRc<DebugArrayUserCurrentExcluded_Cat>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DebugArrayUserCurrentExcluded {
    type Root = DebugArrayUserCurrentExcluded;
    type Parent = DebugArrayUserCurrentExcluded;

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
        *self_rc.array_of_cats.borrow_mut() = Vec::new();
        let l_array_of_cats = 3_usize;
        for _i in 0_usize..l_array_of_cats {
            let t: OptRc<DebugArrayUserCurrentExcluded_Cat> = OptRc::from(DebugArrayUserCurrentExcluded_Cat::default());
            let (root_in, parent_in) = (Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()));
            let root = Self::downcast(root_in, t.clone(), true);
            let parent = Self::downcast(parent_in, t.clone(), false);
            let res = DebugArrayUserCurrentExcluded_Cat::read(&t, &*_io, root, parent);
            self_rc.array_of_cats.borrow_mut().push(t);
            res?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugArrayUserCurrentExcluded {
}
impl DebugArrayUserCurrentExcluded {
    pub fn array_of_cats(&self) -> Ref<'_, Vec<OptRc<DebugArrayUserCurrentExcluded_Cat>>> {
        self.array_of_cats.borrow()
    }
}
impl DebugArrayUserCurrentExcluded {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DebugArrayUserCurrentExcluded_Cat {
    pub(crate) _root: SharedType<DebugArrayUserCurrentExcluded>,
    pub(crate) _parent: SharedType<DebugArrayUserCurrentExcluded>,
    pub(crate) _self_shared: SharedType<Self>,
    meow: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    meow_raw: RefCell<Vec<u8>>,
}
impl KStruct for DebugArrayUserCurrentExcluded_Cat {
    type Root = DebugArrayUserCurrentExcluded;
    type Parent = DebugArrayUserCurrentExcluded;

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
        *self_rc.meow.borrow_mut() = _io.read_bytes(usize::try_from((3_usize).saturating_sub(self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.array_of_cats().len()))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DebugArrayUserCurrentExcluded_Cat {
}
impl DebugArrayUserCurrentExcluded_Cat {
    pub fn meow(&self) -> Ref<'_, Vec<u8>> {
        self.meow.borrow()
    }
}
impl DebugArrayUserCurrentExcluded_Cat {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DebugArrayUserCurrentExcluded_Cat {
    pub fn meow_raw(&self) -> Ref<'_, Vec<u8>> {
        self.meow_raw.borrow()
    }
}
