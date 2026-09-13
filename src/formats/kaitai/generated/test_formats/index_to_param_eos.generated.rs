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
pub struct IndexToParamEos {
    pub(crate) _root: SharedType<IndexToParamEos>,
    pub(crate) _parent: SharedType<IndexToParamEos>,
    pub(crate) _self_shared: SharedType<Self>,
    qty: RefCell<u32>,
    sizes: RefCell<Vec<u32>>,
    blocks: RefCell<Vec<OptRc<IndexToParamEos_Block>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IndexToParamEos {
    type Root = IndexToParamEos;
    type Parent = IndexToParamEos;

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
        *self_rc.blocks.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let f = |t : &mut IndexToParamEos_Block| Ok(t.set_params((_i).try_into().map_err(|_| KError::CastError)?));
                let t = Self::read_into_with_init::<_, IndexToParamEos_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
                self_rc.blocks.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl IndexToParamEos {
}
impl IndexToParamEos {
    pub fn qty(&self) -> Ref<'_, u32> {
        self.qty.borrow()
    }
}
impl IndexToParamEos {
    pub fn sizes(&self) -> Ref<'_, Vec<u32>> {
        self.sizes.borrow()
    }
}
impl IndexToParamEos {
    pub fn blocks(&self) -> Ref<'_, Vec<OptRc<IndexToParamEos_Block>>> {
        self.blocks.borrow()
    }
}
impl IndexToParamEos {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IndexToParamEos_Block {
    pub(crate) _root: SharedType<IndexToParamEos>,
    pub(crate) _parent: SharedType<IndexToParamEos>,
    pub(crate) _self_shared: SharedType<Self>,
    idx: RefCell<i32>,
    buf: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IndexToParamEos_Block {
    type Root = IndexToParamEos;
    type Parent = IndexToParamEos;

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
        *self_rc.buf.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*(self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.sizes().get(usize::try_from(*self_rc.idx())?).ok_or(KError::CastError)?))?)?, "ASCII")?;
        Ok(())
    }
}
impl IndexToParamEos_Block {
    pub fn idx(&self) -> Ref<'_, i32> {
        self.idx.borrow()
    }
}
impl IndexToParamEos_Block {
    pub fn set_params(&mut self, idx: i32) {
        *self.idx.borrow_mut() = idx;
    }
}
impl IndexToParamEos_Block {
}
impl IndexToParamEos_Block {
    pub fn buf(&self) -> Ref<'_, String> {
        self.buf.borrow()
    }
}
impl IndexToParamEos_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
