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
pub struct ValidShort {
    pub(crate) _root: SharedType<ValidShort>,
    pub(crate) _parent: SharedType<ValidShort>,
    pub(crate) _self_shared: SharedType<Self>,
    magic1: RefCell<Vec<u8>>,
    uint8: RefCell<u8>,
    sint8: RefCell<i8>,
    magic_uint: RefCell<String>,
    uint16: RefCell<u16>,
    uint32: RefCell<u32>,
    uint64: RefCell<u64>,
    magic_sint: RefCell<String>,
    sint16: RefCell<i16>,
    sint32: RefCell<i32>,
    sint64: RefCell<i64>,
    _io: RefCell<BytesReader>,
    magic1_raw: RefCell<Vec<u8>>,
    magic_uint_raw: RefCell<Vec<u8>>,
    magic_sint_raw: RefCell<Vec<u8>>,
}
impl KStruct for ValidShort {
    type Root = ValidShort;
    type Parent = ValidShort;

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
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.uint8.borrow_mut() = _io.read_u1()?;
        let expected: u8 = (255).try_into()?;
        if !(*self_rc.uint8() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/1".to_string() }));
        }
        *self_rc.sint8.borrow_mut() = _io.read_s1()?;
        let expected: i8 = (-1).try_into()?;
        if !(*self_rc.sint8() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/2".to_string() }));
        }
        *self_rc.magic_uint.borrow_mut() = bytes_to_str(&_io.read_bytes(10_usize)?, "utf-8")?;
        if !(*self_rc.magic_uint() == "PACK-U-DEF") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/3".to_string() }));
        }
        *self_rc.uint16.borrow_mut() = _io.read_u2le()?;
        let expected: u16 = (65535).try_into()?;
        if !(*self_rc.uint16() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/4".to_string() }));
        }
        *self_rc.uint32.borrow_mut() = _io.read_u4le()?;
        let expected: u32 = (4294967295_i64).try_into()?;
        if !(*self_rc.uint32() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/5".to_string() }));
        }
        *self_rc.uint64.borrow_mut() = _io.read_u8le()?;
        let expected: u64 = (18446744073709551615_i128).try_into()?;
        if !(*self_rc.uint64() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/6".to_string() }));
        }
        *self_rc.magic_sint.borrow_mut() = bytes_to_str(&_io.read_bytes(10_usize)?, "utf-8")?;
        if !(*self_rc.magic_sint() == "PACK-S-DEF") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/7".to_string() }));
        }
        *self_rc.sint16.borrow_mut() = _io.read_s2le()?;
        let expected: i16 = (-1).try_into()?;
        if !(*self_rc.sint16() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/8".to_string() }));
        }
        *self_rc.sint32.borrow_mut() = _io.read_s4le()?;
        let expected: i32 = (-1).try_into()?;
        if !(*self_rc.sint32() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/9".to_string() }));
        }
        *self_rc.sint64.borrow_mut() = _io.read_s8le()?;
        let expected: i64 = (-1).try_into()?;
        if !(*self_rc.sint64() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/10".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ValidShort {
}
impl ValidShort {
    pub fn magic1(&self) -> Ref<'_, Vec<u8>> {
        self.magic1.borrow()
    }
}
impl ValidShort {
    pub fn uint8(&self) -> Ref<'_, u8> {
        self.uint8.borrow()
    }
}
impl ValidShort {
    pub fn sint8(&self) -> Ref<'_, i8> {
        self.sint8.borrow()
    }
}
impl ValidShort {
    pub fn magic_uint(&self) -> Ref<'_, String> {
        self.magic_uint.borrow()
    }
}
impl ValidShort {
    pub fn uint16(&self) -> Ref<'_, u16> {
        self.uint16.borrow()
    }
}
impl ValidShort {
    pub fn uint32(&self) -> Ref<'_, u32> {
        self.uint32.borrow()
    }
}
impl ValidShort {
    pub fn uint64(&self) -> Ref<'_, u64> {
        self.uint64.borrow()
    }
}
impl ValidShort {
    pub fn magic_sint(&self) -> Ref<'_, String> {
        self.magic_sint.borrow()
    }
}
impl ValidShort {
    pub fn sint16(&self) -> Ref<'_, i16> {
        self.sint16.borrow()
    }
}
impl ValidShort {
    pub fn sint32(&self) -> Ref<'_, i32> {
        self.sint32.borrow()
    }
}
impl ValidShort {
    pub fn sint64(&self) -> Ref<'_, i64> {
        self.sint64.borrow()
    }
}
impl ValidShort {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ValidShort {
    pub fn magic1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.magic1_raw.borrow()
    }
}
impl ValidShort {
    pub fn magic_uint_raw(&self) -> Ref<'_, Vec<u8>> {
        self.magic_uint_raw.borrow()
    }
}
impl ValidShort {
    pub fn magic_sint_raw(&self) -> Ref<'_, Vec<u8>> {
        self.magic_sint_raw.borrow()
    }
}
