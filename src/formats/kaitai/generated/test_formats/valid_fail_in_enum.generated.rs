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
pub struct ValidFailInEnum {
    pub(crate) _root: SharedType<ValidFailInEnum>,
    pub(crate) _parent: SharedType<ValidFailInEnum>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<ValidFailInEnum_Animal>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ValidFailInEnum {
    type Root = ValidFailInEnum;
    type Parent = ValidFailInEnum;

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
        *self_rc.foo.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        if matches!(*self_rc.foo(), ValidFailInEnum_Animal::Unknown(_)) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotInEnum, src_path: "/seq/0".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ValidFailInEnum {
}
impl ValidFailInEnum {
    pub fn foo(&self) -> Ref<'_, ValidFailInEnum_Animal> {
        self.foo.borrow()
    }
}
impl ValidFailInEnum {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ValidFailInEnum_Animal {
    Dog,
    Chicken,
    Unknown(i64),
}

impl TryFrom<i64> for ValidFailInEnum_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ValidFailInEnum_Animal> {
        match flag {
            4 => Ok(ValidFailInEnum_Animal::Dog),
            12 => Ok(ValidFailInEnum_Animal::Chicken),
            _ => Ok(ValidFailInEnum_Animal::Unknown(flag)),
        }
    }
}

impl From<&ValidFailInEnum_Animal> for i64 {
    fn from(v: &ValidFailInEnum_Animal) -> Self {
        match *v {
            ValidFailInEnum_Animal::Dog => 4,
            ValidFailInEnum_Animal::Chicken => 12,
            ValidFailInEnum_Animal::Unknown(v) => v
        }
    }
}

impl Default for ValidFailInEnum_Animal {
    fn default() -> Self { ValidFailInEnum_Animal::Unknown(0) }
}

