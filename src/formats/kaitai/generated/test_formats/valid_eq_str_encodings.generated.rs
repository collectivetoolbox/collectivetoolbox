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
pub struct ValidEqStrEncodings {
    pub(crate) _root: SharedType<ValidEqStrEncodings>,
    pub(crate) _parent: SharedType<ValidEqStrEncodings>,
    pub(crate) _self_shared: SharedType<Self>,
    len_of_1: RefCell<u16>,
    str1: RefCell<String>,
    len_of_2: RefCell<u16>,
    str2: RefCell<String>,
    len_of_3: RefCell<u16>,
    str3: RefCell<String>,
    len_of_4: RefCell<u16>,
    str4: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ValidEqStrEncodings {
    type Root = ValidEqStrEncodings;
    type Parent = ValidEqStrEncodings;

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
        *self_rc.len_of_1.borrow_mut() = _io.read_u2le()?;
        *self_rc.str1.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_1()))?, "ASCII")?;
        if !(*self_rc.str1() == "Some ASCII") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/1".to_string() }));
        }
        *self_rc.len_of_2.borrow_mut() = _io.read_u2le()?;
        *self_rc.str2.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_2()))?, "UTF-8")?;
        if !(*self_rc.str2() == "ã\u{81}\u{93}ã\u{82}\u{93}ã\u{81}«ã\u{81}¡ã\u{81}¯") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/3".to_string() }));
        }
        *self_rc.len_of_3.borrow_mut() = _io.read_u2le()?;
        *self_rc.str3.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_3()))?, "SJIS")?;
        if !(*self_rc.str3() == "ã\u{81}\u{93}ã\u{82}\u{93}ã\u{81}«ã\u{81}¡ã\u{81}¯") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/5".to_string() }));
        }
        *self_rc.len_of_4.borrow_mut() = _io.read_u2le()?;
        *self_rc.str4.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_4()))?, "IBM437")?;
        if !(*self_rc.str4() == "â\u{96}\u{91}â\u{96}\u{92}â\u{96}\u{93}") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/7".to_string() }));
        }
        Ok(())
    }
}
impl ValidEqStrEncodings {
}
impl ValidEqStrEncodings {
    pub fn len_of_1(&self) -> Ref<'_, u16> {
        self.len_of_1.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn str1(&self) -> Ref<'_, String> {
        self.str1.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn len_of_2(&self) -> Ref<'_, u16> {
        self.len_of_2.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn str2(&self) -> Ref<'_, String> {
        self.str2.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn len_of_3(&self) -> Ref<'_, u16> {
        self.len_of_3.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn str3(&self) -> Ref<'_, String> {
        self.str3.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn len_of_4(&self) -> Ref<'_, u16> {
        self.len_of_4.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn str4(&self) -> Ref<'_, String> {
        self.str4.borrow()
    }
}
impl ValidEqStrEncodings {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
