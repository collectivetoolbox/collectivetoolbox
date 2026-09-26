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
pub struct FixedStruct {
    pub(crate) _root: SharedType<FixedStruct>,
    pub(crate) _parent: SharedType<FixedStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_hdr: Cell<bool>,
    hdr: RefCell<OptRc<FixedStruct_Header>>,
}
impl KStruct for FixedStruct {
    type Root = FixedStruct;
    type Parent = FixedStruct;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl FixedStruct {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn hdr(
        &self
    ) -> KResult<Ref<'_, OptRc<FixedStruct_Header>>> {
        let _io = self._io.borrow();
        if self.f_hdr.get() {
            return Ok(self.hdr.borrow());
        }
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        let t = Self::read_into::<_, FixedStruct_Header>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.hdr.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.hdr.borrow())
    }
}
impl FixedStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FixedStruct_Header {
    pub(crate) _root: SharedType<FixedStruct>,
    pub(crate) _parent: SharedType<FixedStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    magic1: RefCell<Vec<u8>>,
    uint8: RefCell<u8>,
    sint8: RefCell<i8>,
    magic_uint: RefCell<Vec<u8>>,
    uint16: RefCell<u16>,
    uint32: RefCell<u32>,
    uint64: RefCell<u64>,
    magic_sint: RefCell<Vec<u8>>,
    sint16: RefCell<i16>,
    sint32: RefCell<i32>,
    sint64: RefCell<i64>,
    magic_uint_le: RefCell<Vec<u8>>,
    uint16le: RefCell<u16>,
    uint32le: RefCell<u32>,
    uint64le: RefCell<u64>,
    magic_sint_le: RefCell<Vec<u8>>,
    sint16le: RefCell<i16>,
    sint32le: RefCell<i32>,
    sint64le: RefCell<i64>,
    magic_uint_be: RefCell<Vec<u8>>,
    uint16be: RefCell<u16>,
    uint32be: RefCell<u32>,
    uint64be: RefCell<u64>,
    magic_sint_be: RefCell<Vec<u8>>,
    sint16be: RefCell<i16>,
    sint32be: RefCell<i32>,
    sint64be: RefCell<i64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FixedStruct_Header {
    type Root = FixedStruct;
    type Parent = FixedStruct;

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
        *self_rc.magic1.borrow_mut() = _io.read_bytes(6_usize)?;
        if !(*self_rc.magic1() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x31u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/0".to_string() }));
        }
        *self_rc.uint8.borrow_mut() = _io.read_u1()?;
        *self_rc.sint8.borrow_mut() = _io.read_s1()?;
        *self_rc.magic_uint.borrow_mut() = _io.read_bytes(10_usize)?;
        if !(*self_rc.magic_uint() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x55u8, 0x2du8, 0x44u8, 0x45u8, 0x46u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/3".to_string() }));
        }
        *self_rc.uint16.borrow_mut() = _io.read_u2le()?;
        *self_rc.uint32.borrow_mut() = _io.read_u4le()?;
        *self_rc.uint64.borrow_mut() = _io.read_u8le()?;
        *self_rc.magic_sint.borrow_mut() = _io.read_bytes(10_usize)?;
        if !(*self_rc.magic_sint() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x53u8, 0x2du8, 0x44u8, 0x45u8, 0x46u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/7".to_string() }));
        }
        *self_rc.sint16.borrow_mut() = _io.read_s2le()?;
        *self_rc.sint32.borrow_mut() = _io.read_s4le()?;
        *self_rc.sint64.borrow_mut() = _io.read_s8le()?;
        *self_rc.magic_uint_le.borrow_mut() = _io.read_bytes(9_usize)?;
        if !(*self_rc.magic_uint_le() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x55u8, 0x2du8, 0x4cu8, 0x45u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/11".to_string() }));
        }
        *self_rc.uint16le.borrow_mut() = _io.read_u2le()?;
        *self_rc.uint32le.borrow_mut() = _io.read_u4le()?;
        *self_rc.uint64le.borrow_mut() = _io.read_u8le()?;
        *self_rc.magic_sint_le.borrow_mut() = _io.read_bytes(9_usize)?;
        if !(*self_rc.magic_sint_le() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x53u8, 0x2du8, 0x4cu8, 0x45u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/15".to_string() }));
        }
        *self_rc.sint16le.borrow_mut() = _io.read_s2le()?;
        *self_rc.sint32le.borrow_mut() = _io.read_s4le()?;
        *self_rc.sint64le.borrow_mut() = _io.read_s8le()?;
        *self_rc.magic_uint_be.borrow_mut() = _io.read_bytes(9_usize)?;
        if !(*self_rc.magic_uint_be() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x55u8, 0x2du8, 0x42u8, 0x45u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/19".to_string() }));
        }
        *self_rc.uint16be.borrow_mut() = _io.read_u2be()?;
        *self_rc.uint32be.borrow_mut() = _io.read_u4be()?;
        *self_rc.uint64be.borrow_mut() = _io.read_u8be()?;
        *self_rc.magic_sint_be.borrow_mut() = _io.read_bytes(9_usize)?;
        if !(*self_rc.magic_sint_be() == vec![0x50u8, 0x41u8, 0x43u8, 0x4bu8, 0x2du8, 0x53u8, 0x2du8, 0x42u8, 0x45u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/23".to_string() }));
        }
        *self_rc.sint16be.borrow_mut() = _io.read_s2be()?;
        *self_rc.sint32be.borrow_mut() = _io.read_s4be()?;
        *self_rc.sint64be.borrow_mut() = _io.read_s8be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl FixedStruct_Header {
}
impl FixedStruct_Header {
    pub fn magic1(&self) -> Ref<'_, Vec<u8>> {
        self.magic1.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint8(&self) -> Ref<'_, u8> {
        self.uint8.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint8(&self) -> Ref<'_, i8> {
        self.sint8.borrow()
    }
}
impl FixedStruct_Header {
    pub fn magic_uint(&self) -> Ref<'_, Vec<u8>> {
        self.magic_uint.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint16(&self) -> Ref<'_, u16> {
        self.uint16.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint32(&self) -> Ref<'_, u32> {
        self.uint32.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint64(&self) -> Ref<'_, u64> {
        self.uint64.borrow()
    }
}
impl FixedStruct_Header {
    pub fn magic_sint(&self) -> Ref<'_, Vec<u8>> {
        self.magic_sint.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint16(&self) -> Ref<'_, i16> {
        self.sint16.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint32(&self) -> Ref<'_, i32> {
        self.sint32.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint64(&self) -> Ref<'_, i64> {
        self.sint64.borrow()
    }
}
impl FixedStruct_Header {
    pub fn magic_uint_le(&self) -> Ref<'_, Vec<u8>> {
        self.magic_uint_le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint16le(&self) -> Ref<'_, u16> {
        self.uint16le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint32le(&self) -> Ref<'_, u32> {
        self.uint32le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint64le(&self) -> Ref<'_, u64> {
        self.uint64le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn magic_sint_le(&self) -> Ref<'_, Vec<u8>> {
        self.magic_sint_le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint16le(&self) -> Ref<'_, i16> {
        self.sint16le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint32le(&self) -> Ref<'_, i32> {
        self.sint32le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint64le(&self) -> Ref<'_, i64> {
        self.sint64le.borrow()
    }
}
impl FixedStruct_Header {
    pub fn magic_uint_be(&self) -> Ref<'_, Vec<u8>> {
        self.magic_uint_be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint16be(&self) -> Ref<'_, u16> {
        self.uint16be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint32be(&self) -> Ref<'_, u32> {
        self.uint32be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn uint64be(&self) -> Ref<'_, u64> {
        self.uint64be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn magic_sint_be(&self) -> Ref<'_, Vec<u8>> {
        self.magic_sint_be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint16be(&self) -> Ref<'_, i16> {
        self.sint16be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint32be(&self) -> Ref<'_, i32> {
        self.sint32be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn sint64be(&self) -> Ref<'_, i64> {
        self.sint64be.borrow()
    }
}
impl FixedStruct_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
