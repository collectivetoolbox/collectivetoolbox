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

/**
 * Another one-liner
 * \sa <http://www.example.com/some/path/?even_with=query&more=2> Source
 */

#[derive(Default, Debug, Clone)]
pub struct DocstringsDocref {
    pub(crate) _root: SharedType<DocstringsDocref>,
    pub(crate) _parent: SharedType<DocstringsDocref>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u8>,
    two: RefCell<u8>,
    three: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_foo: Cell<bool>,
    foo: RefCell<bool>,
    f_parse_inst: Cell<bool>,
    parse_inst: RefCell<u8>,
}
impl KStruct for DocstringsDocref {
    type Root = DocstringsDocref;
    type Parent = DocstringsDocref;

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
        *self_rc.one.borrow_mut() = _io.read_u1()?;
        *self_rc.two.borrow_mut() = _io.read_u1()?;
        *self_rc.three.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DocstringsDocref {

    /**
     * \sa Doc ref for instance, a plain one
     */
    pub fn foo(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_foo.get() {
            return Ok(self.foo.borrow());
        }
        self.f_foo.set(true);
        *self.foo.borrow_mut() = (true).try_into()?;
        Ok(self.foo.borrow())
    }

    /**
     * \sa Now this is a really
     *   long document ref that
     *   spans multiple lines.
     */
    pub fn parse_inst(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_parse_inst.get() {
            return Ok(self.parse_inst.borrow());
        }
        self.f_parse_inst.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.parse_inst.borrow_mut() = _io.read_u1()?;
        _io.seek(_pos)?;
        Ok(self.parse_inst.borrow())
    }
}

/**
 * \sa Plain text description of doc ref, page 42
 */
impl DocstringsDocref {
    pub fn one(&self) -> Ref<'_, u8> {
        self.one.borrow()
    }
}

/**
 * Both doc and doc-ref are defined
 * \sa <http://www.example.com/with/url/again> Source
 */
impl DocstringsDocref {
    pub fn two(&self) -> Ref<'_, u8> {
        self.two.borrow()
    }
}

/**
 * \sa http://www.example.com/three Documentation name
 */
impl DocstringsDocref {
    pub fn three(&self) -> Ref<'_, u8> {
        self.three.borrow()
    }
}
impl DocstringsDocref {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
