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
 * One-liner description of a type.
 */

#[derive(Default, Debug, Clone)]
pub struct Docstrings {
    pub(crate) _root: SharedType<Docstrings>,
    pub(crate) _parent: SharedType<Docstrings>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_three: Cell<bool>,
    three: RefCell<i32>,
    f_two: Cell<bool>,
    two: RefCell<u8>,
}
impl KStruct for Docstrings {
    type Root = Docstrings;
    type Parent = Docstrings;

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
        *self_rc.one.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Docstrings {

    /**
     * And yet another one for value instance "three"
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn three(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_three.get() {
            return Ok(self.three.borrow());
        }
        self.f_three.set(true);
        *self.three.borrow_mut() = (66).try_into()?;
        Ok(self.three.borrow())
    }

    /**
     * Another description for parse instance "two"
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_two.get() {
            return Ok(self.two.borrow());
        }
        self.f_two.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.two.borrow_mut() = _io.read_u1()?;
        _io.seek(_pos)?;
        Ok(self.two.borrow())
    }
}

/**
 * A pretty verbose description for sequence attribute "one"
 */
impl Docstrings {
    pub fn one(&self) -> Ref<'_, u8> {
        self.one.borrow()
    }
}
impl Docstrings {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * This subtype is never used, yet has a very long description
 * that spans multiple lines. It should be formatted accordingly,
 * even in languages that have single-line comments for
 * docstrings. Actually, there's even a MarkDown-style list here
 * with several bullets:
 *
 * * one
 * * two
 * * three
 *
 * And the text continues after that. Here's a MarkDown-style link:
 * [woohoo](http://example.com) - one day it will be supported as
 * well.
 */

#[derive(Default, Debug, Clone)]
pub struct Docstrings_ComplexSubtype {
    pub(crate) _root: SharedType<Docstrings>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Docstrings_ComplexSubtype {
    type Root = Docstrings;
    type Parent = KStructUnit;

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
impl Docstrings_ComplexSubtype {
}
impl Docstrings_ComplexSubtype {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
