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
pub struct ValidSwitch {
    pub(crate) _root: SharedType<ValidSwitch>,
    pub(crate) _parent: SharedType<ValidSwitch>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<u8>,
    b: RefCell<Option<ValidSwitch_B>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum ValidSwitch_B {
    U2(u16),
}
impl From<u16> for ValidSwitch_B {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl TryFrom<&ValidSwitch_B> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &ValidSwitch_B) -> Result<Self, Self::Error> {
        match e {
            ValidSwitch_B::U2(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&ValidSwitch_B> for u16 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &ValidSwitch_B) -> Result<Self, Self::Error> {
        match e {
            ValidSwitch_B::U2(v) => Ok(u16::try_from(*v)?),
        }
    }
}
impl TryFrom<&ValidSwitch_B> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &ValidSwitch_B) -> Result<Self, Self::Error> {
        match e {
            ValidSwitch_B::U2(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&ValidSwitch_B> for usize {
    type Error = KError;
    fn try_from(e: &ValidSwitch_B) -> Result<Self, Self::Error> {
        match e {
            ValidSwitch_B::U2(v) => Ok(usize::from(*v)),
        }
    }
}

impl KStruct for ValidSwitch {
    type Root = ValidSwitch;
    type Parent = ValidSwitch;

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
        *self_rc.a.borrow_mut() = _io.read_u1()?;
        let expected: u8 = (80).try_into()?;
        if !(*self_rc.a() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        match *self_rc.a() {
            80 => {
                *self_rc.b.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            _ => {
                *self_rc.b.borrow_mut() = Some(_io.read_u2be()?.into());
            }
        }
        let expected: u16 = (17217).try_into()?;
        if !(self_rc.b() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/1".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ValidSwitch {
}
impl ValidSwitch {
    pub fn a(&self) -> Ref<'_, u8> {
        self.a.borrow()
    }
}
impl ValidSwitch {
    pub fn b(&self) -> u16 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.b.borrow().as_ref().and_then(|v| u16::try_from(v).ok()).unwrap_or(0)
    }
    pub fn b_enum(&self) -> Ref<'_, Option<ValidSwitch_B>> {
        self.b.borrow()
    }
}
impl ValidSwitch {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
