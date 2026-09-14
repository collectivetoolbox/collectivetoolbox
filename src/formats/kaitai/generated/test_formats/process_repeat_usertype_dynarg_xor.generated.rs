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
pub struct ProcessRepeatUsertypeDynargXor {
    pub(crate) _root: SharedType<ProcessRepeatUsertypeDynargXor>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertypeDynargXor>,
    pub(crate) _self_shared: SharedType<Self>,
    blocks: RefCell<Vec<OptRc<ProcessRepeatUsertypeDynargXor_Block>>>,
    blocks_b: RefCell<OptRc<ProcessRepeatUsertypeDynargXor_BlocksBWrapper>>,
    _io: RefCell<BytesReader>,
    blocks_raw: RefCell<Vec<u8>>,
}
impl KStruct for ProcessRepeatUsertypeDynargXor {
    type Root = ProcessRepeatUsertypeDynargXor;
    type Parent = ProcessRepeatUsertypeDynargXor;

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
        *self_rc.blocks.borrow_mut() = Vec::new();
        let l_blocks = 2_usize;
        for _i in 0_usize..l_blocks {
            let _raw_blocks = _io.read_bytes(5_usize)?;
            let _processed_blocks = process_xor_one(&_raw_blocks, u8::try_from(i64::try_from(((155_usize) ^ (((usize::try_from((u64::try_from(_i)?).wrapping_shl(4_u32))?) | (_io.pos())))))? & 0xff)?);
            let _io_blocks = BytesReader::from(_processed_blocks);
            let t = Self::read_into::<BytesReader, ProcessRepeatUsertypeDynargXor_Block>(&_io_blocks, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.blocks.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ProcessRepeatUsertypeDynargXor_BlocksBWrapper>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.blocks_b.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertypeDynargXor {
}
impl ProcessRepeatUsertypeDynargXor {
    pub fn blocks(&self) -> Ref<'_, Vec<OptRc<ProcessRepeatUsertypeDynargXor_Block>>> {
        self.blocks.borrow()
    }
}
impl ProcessRepeatUsertypeDynargXor {
    pub fn blocks_b(&self) -> Ref<'_, OptRc<ProcessRepeatUsertypeDynargXor_BlocksBWrapper>> {
        self.blocks_b.borrow()
    }
}
impl ProcessRepeatUsertypeDynargXor {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessRepeatUsertypeDynargXor {
    pub fn blocks_raw(&self) -> Ref<'_, Vec<u8>> {
        self.blocks_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessRepeatUsertypeDynargXor_Block {
    pub(crate) _root: SharedType<ProcessRepeatUsertypeDynargXor>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertypeDynargXor>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ProcessRepeatUsertypeDynargXor_Block {
    type Root = ProcessRepeatUsertypeDynargXor;
    type Parent = ProcessRepeatUsertypeDynargXor;

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
        *self_rc.a.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertypeDynargXor_Block {
}
impl ProcessRepeatUsertypeDynargXor_Block {
    pub fn a(&self) -> Ref<'_, u32> {
        self.a.borrow()
    }
}
impl ProcessRepeatUsertypeDynargXor_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessRepeatUsertypeDynargXor_BlocksBWrapper {
    pub(crate) _root: SharedType<ProcessRepeatUsertypeDynargXor>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertypeDynargXor>,
    pub(crate) _self_shared: SharedType<Self>,
    dummy: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_blocks_0_b: Cell<bool>,
    blocks_0_b: RefCell<u8>,
    f_blocks_1_b: Cell<bool>,
    blocks_1_b: RefCell<u8>,
}
impl KStruct for ProcessRepeatUsertypeDynargXor_BlocksBWrapper {
    type Root = ProcessRepeatUsertypeDynargXor;
    type Parent = ProcessRepeatUsertypeDynargXor;

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
        *self_rc.dummy.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertypeDynargXor_BlocksBWrapper {
    pub fn blocks_0_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_0_b.get() {
            return Ok(self.blocks_0_b.borrow());
        }
        self.f_blocks_0_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks().get(0_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(4_usize)?;
        *self.blocks_0_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_0_b.borrow())
    }
    pub fn blocks_1_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_1_b.get() {
            return Ok(self.blocks_1_b.borrow());
        }
        self.f_blocks_1_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks().get(1_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(4_usize)?;
        *self.blocks_1_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_1_b.borrow())
    }
}
impl ProcessRepeatUsertypeDynargXor_BlocksBWrapper {
    pub fn dummy(&self) -> Ref<'_, u8> {
        self.dummy.borrow()
    }
}
impl ProcessRepeatUsertypeDynargXor_BlocksBWrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
