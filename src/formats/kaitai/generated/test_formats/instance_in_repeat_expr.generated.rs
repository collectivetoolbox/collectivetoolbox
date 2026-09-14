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
pub struct InstanceInRepeatExpr {
    pub(crate) _root: SharedType<InstanceInRepeatExpr>,
    pub(crate) _parent: SharedType<InstanceInRepeatExpr>,
    pub(crate) _self_shared: SharedType<Self>,
    chunks: RefCell<Vec<OptRc<InstanceInRepeatExpr_Chunk>>>,
    _io: RefCell<BytesReader>,
    f_num_chunks: Cell<bool>,
    num_chunks: RefCell<u32>,
}
impl KStruct for InstanceInRepeatExpr {
    type Root = InstanceInRepeatExpr;
    type Parent = InstanceInRepeatExpr;

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
        *self_rc.chunks.borrow_mut() = Vec::new();
        let l_chunks = usize::try_from(*self_rc.num_chunks()?)?;
        for _i in 0_usize..l_chunks {
            let t = Self::read_into::<_, InstanceInRepeatExpr_Chunk>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.chunks.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInRepeatExpr {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn num_chunks(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_num_chunks.get() {
            return Ok(self.num_chunks.borrow());
        }
        self.f_num_chunks.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((_io.pos()).saturating_add(16_usize))?)?;
        *self.num_chunks.borrow_mut() = _io.read_u4le()?;
        _io.seek(_pos)?;
        Ok(self.num_chunks.borrow())
    }
}
impl InstanceInRepeatExpr {
    pub fn chunks(&self) -> Ref<'_, Vec<OptRc<InstanceInRepeatExpr_Chunk>>> {
        self.chunks.borrow()
    }
}
impl InstanceInRepeatExpr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceInRepeatExpr_Chunk {
    pub(crate) _root: SharedType<InstanceInRepeatExpr>,
    pub(crate) _parent: SharedType<InstanceInRepeatExpr>,
    pub(crate) _self_shared: SharedType<Self>,
    offset: RefCell<u32>,
    len: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for InstanceInRepeatExpr_Chunk {
    type Root = InstanceInRepeatExpr;
    type Parent = InstanceInRepeatExpr;

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
        *self_rc.offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.len.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInRepeatExpr_Chunk {
}
impl InstanceInRepeatExpr_Chunk {
    pub fn offset(&self) -> Ref<'_, u32> {
        self.offset.borrow()
    }
}
impl InstanceInRepeatExpr_Chunk {
    pub fn len(&self) -> Ref<'_, u32> {
        self.len.borrow()
    }
}
impl InstanceInRepeatExpr_Chunk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
