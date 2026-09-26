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
pub struct ProcessRepeatUsertype {
    pub(crate) _root: SharedType<ProcessRepeatUsertype>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    blocks: RefCell<Vec<OptRc<ProcessRepeatUsertype_Block>>>,
    _io: RefCell<BytesReader>,
    blocks_raw: RefCell<Vec<u8>>,
}
impl KStruct for ProcessRepeatUsertype {
    type Root = ProcessRepeatUsertype;
    type Parent = ProcessRepeatUsertype;

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
        *self_rc.blocks.borrow_mut() = Vec::new();
        let l_blocks = 2_usize;
        for _i in 0_usize..l_blocks {
            let _raw_blocks = _io.read_bytes(5_usize)?;
            let _processed_blocks = process_xor_one(&_raw_blocks, 158_u8);
            let _io_blocks = BytesReader::from(_processed_blocks);
            let t = Self::read_into::<BytesReader, ProcessRepeatUsertype_Block>(&_io_blocks, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.blocks.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertype {
}
impl ProcessRepeatUsertype {
    pub fn blocks(&self) -> Ref<'_, Vec<OptRc<ProcessRepeatUsertype_Block>>> {
        self.blocks.borrow()
    }
}
impl ProcessRepeatUsertype {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessRepeatUsertype {
    pub fn blocks_raw(&self) -> Ref<'_, Vec<u8>> {
        self.blocks_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessRepeatUsertype_Block {
    pub(crate) _root: SharedType<ProcessRepeatUsertype>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<i32>,
    b: RefCell<i8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ProcessRepeatUsertype_Block {
    type Root = ProcessRepeatUsertype;
    type Parent = ProcessRepeatUsertype;

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
        *self_rc.a.borrow_mut() = _io.read_s4le()?;
        *self_rc.b.borrow_mut() = _io.read_s1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertype_Block {
}
impl ProcessRepeatUsertype_Block {
    pub fn a(&self) -> Ref<'_, i32> {
        self.a.borrow()
    }
}
impl ProcessRepeatUsertype_Block {
    pub fn b(&self) -> Ref<'_, i8> {
        self.b.borrow()
    }
}
impl ProcessRepeatUsertype_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
