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
pub struct IndexSizes {
    pub(crate) _root: SharedType<IndexSizes>,
    pub(crate) _parent: SharedType<IndexSizes>,
    pub(crate) _self_shared: SharedType<Self>,
    qty: RefCell<u32>,
    sizes: RefCell<Vec<u32>>,
    bufs: RefCell<Vec<String>>,
    _io: RefCell<BytesReader>,
    bufs_raw: RefCell<Vec<u8>>,
}
impl KStruct for IndexSizes {
    type Root = IndexSizes;
    type Parent = IndexSizes;

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
        *self_rc.qty.borrow_mut() = _io.read_u4le()?;
        *self_rc.sizes.borrow_mut() = Vec::new();
        let l_sizes = usize::try_from(*self_rc.qty())?;
        for _i in 0_usize..l_sizes {
            self_rc.sizes.borrow_mut().push(_io.read_u4le()?);
        }
        *self_rc.bufs.borrow_mut() = Vec::new();
        let l_bufs = usize::try_from(*self_rc.qty())?;
        for _i in 0_usize..l_bufs {
            self_rc.bufs.borrow_mut().push(bytes_to_str(&_io.read_bytes(usize::try_from(*(self_rc.sizes().get(_i).ok_or(KError::CastError)?))?)?, "ASCII")?);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IndexSizes {
}
impl IndexSizes {
    pub fn qty(&self) -> Ref<'_, u32> {
        self.qty.borrow()
    }
}
impl IndexSizes {
    pub fn sizes(&self) -> Ref<'_, Vec<u32>> {
        self.sizes.borrow()
    }
}
impl IndexSizes {
    pub fn bufs(&self) -> Ref<'_, Vec<String>> {
        self.bufs.borrow()
    }
}
impl IndexSizes {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl IndexSizes {
    pub fn bufs_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bufs_raw.borrow()
    }
}
