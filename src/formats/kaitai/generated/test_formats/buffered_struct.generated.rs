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
pub struct BufferedStruct {
    pub(crate) _root: SharedType<BufferedStruct>,
    pub(crate) _parent: SharedType<BufferedStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    len1: RefCell<u32>,
    block1: RefCell<OptRc<BufferedStruct_Block>>,
    len2: RefCell<u32>,
    block2: RefCell<OptRc<BufferedStruct_Block>>,
    finisher: RefCell<u32>,
    _io: RefCell<BytesReader>,
    block1_raw: RefCell<Vec<u8>>,
    block2_raw: RefCell<Vec<u8>>,
}
impl KStruct for BufferedStruct {
    type Root = BufferedStruct;
    type Parent = BufferedStruct;

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
        *self_rc.len1.borrow_mut() = _io.read_u4le()?;
        let _raw_block1 = _io.read_bytes(usize::try_from(*self_rc.len1())?)?;
        *self_rc.block1_raw.borrow_mut() = _raw_block1.clone();
        let _io_block1 = BytesReader::from(_raw_block1);
        let t = Self::read_into::<BytesReader, BufferedStruct_Block>(&_io_block1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.block1.borrow_mut() = t;
        *self_rc.len2.borrow_mut() = _io.read_u4le()?;
        let _raw_block2 = _io.read_bytes(usize::try_from(*self_rc.len2())?)?;
        *self_rc.block2_raw.borrow_mut() = _raw_block2.clone();
        let _io_block2 = BytesReader::from(_raw_block2);
        let t = Self::read_into::<BytesReader, BufferedStruct_Block>(&_io_block2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.block2.borrow_mut() = t;
        *self_rc.finisher.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BufferedStruct {
}
impl BufferedStruct {
    pub fn len1(&self) -> Ref<'_, u32> {
        self.len1.borrow()
    }
}
impl BufferedStruct {
    pub fn block1(&self) -> Ref<'_, OptRc<BufferedStruct_Block>> {
        self.block1.borrow()
    }
}
impl BufferedStruct {
    pub fn len2(&self) -> Ref<'_, u32> {
        self.len2.borrow()
    }
}
impl BufferedStruct {
    pub fn block2(&self) -> Ref<'_, OptRc<BufferedStruct_Block>> {
        self.block2.borrow()
    }
}
impl BufferedStruct {
    pub fn finisher(&self) -> Ref<'_, u32> {
        self.finisher.borrow()
    }
}
impl BufferedStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl BufferedStruct {
    pub fn block1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.block1_raw.borrow()
    }
}
impl BufferedStruct {
    pub fn block2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.block2_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BufferedStruct_Block {
    pub(crate) _root: SharedType<BufferedStruct>,
    pub(crate) _parent: SharedType<BufferedStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    number1: RefCell<u32>,
    number2: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BufferedStruct_Block {
    type Root = BufferedStruct;
    type Parent = BufferedStruct;

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
        *self_rc.number1.borrow_mut() = _io.read_u4le()?;
        *self_rc.number2.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BufferedStruct_Block {
}
impl BufferedStruct_Block {
    pub fn number1(&self) -> Ref<'_, u32> {
        self.number1.borrow()
    }
}
impl BufferedStruct_Block {
    pub fn number2(&self) -> Ref<'_, u32> {
        self.number2.borrow()
    }
}
impl BufferedStruct_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
