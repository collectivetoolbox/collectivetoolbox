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
pub struct ProcessCoerceSwitch {
    pub(crate) _root: SharedType<ProcessCoerceSwitch>,
    pub(crate) _parent: SharedType<ProcessCoerceSwitch>,
    pub(crate) _self_shared: SharedType<Self>,
    buf_type: RefCell<u8>,
    flag: RefCell<u8>,
    buf_unproc: RefCell<Option<ProcessCoerceSwitch_BufUnproc>>,
    buf_proc: RefCell<Option<ProcessCoerceSwitch_BufProc>>,
    _io: RefCell<BytesReader>,
    buf_unproc_raw: RefCell<Vec<u8>>,
    buf_proc_raw: RefCell<Vec<u8>>,
    f_buf: Cell<bool>,
    buf: RefCell<OptRc<ProcessCoerceSwitch_Foo>>,
}
#[derive(Debug, Clone)]
pub enum ProcessCoerceSwitch_BufUnproc {
    ProcessCoerceSwitch_Foo(OptRc<ProcessCoerceSwitch_Foo>),
    Bytes(Vec<u8>),
}
impl TryFrom<&ProcessCoerceSwitch_BufUnproc> for OptRc<ProcessCoerceSwitch_Foo> {
    type Error = KError;
    fn try_from(v: &ProcessCoerceSwitch_BufUnproc) -> Result<Self, Self::Error> {
        if let ProcessCoerceSwitch_BufUnproc::ProcessCoerceSwitch_Foo(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<ProcessCoerceSwitch_Foo>> for ProcessCoerceSwitch_BufUnproc {
    fn from(v: OptRc<ProcessCoerceSwitch_Foo>) -> Self {
        Self::ProcessCoerceSwitch_Foo(v)
    }
}
impl TryFrom<&ProcessCoerceSwitch_BufUnproc> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &ProcessCoerceSwitch_BufUnproc) -> Result<Self, Self::Error> {
        if let ProcessCoerceSwitch_BufUnproc::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for ProcessCoerceSwitch_BufUnproc {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
#[derive(Debug, Clone)]
pub enum ProcessCoerceSwitch_BufProc {
    ProcessCoerceSwitch_Foo(OptRc<ProcessCoerceSwitch_Foo>),
    Bytes(Vec<u8>),
}
impl TryFrom<&ProcessCoerceSwitch_BufProc> for OptRc<ProcessCoerceSwitch_Foo> {
    type Error = KError;
    fn try_from(v: &ProcessCoerceSwitch_BufProc) -> Result<Self, Self::Error> {
        if let ProcessCoerceSwitch_BufProc::ProcessCoerceSwitch_Foo(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<ProcessCoerceSwitch_Foo>> for ProcessCoerceSwitch_BufProc {
    fn from(v: OptRc<ProcessCoerceSwitch_Foo>) -> Self {
        Self::ProcessCoerceSwitch_Foo(v)
    }
}
impl TryFrom<&ProcessCoerceSwitch_BufProc> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &ProcessCoerceSwitch_BufProc) -> Result<Self, Self::Error> {
        if let ProcessCoerceSwitch_BufProc::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for ProcessCoerceSwitch_BufProc {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for ProcessCoerceSwitch {
    type Root = ProcessCoerceSwitch;
    type Parent = ProcessCoerceSwitch;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.buf_type.borrow_mut() = _io.read_u1()?;
        *self_rc.flag.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.flag())) == (to_i128(0))) {
            match *self_rc.buf_type() {
                0 => {
                    *self_rc.buf_unproc_raw.borrow_mut() = _io.read_bytes(4_usize)?.into();
                    let buf_unproc_raw = self_rc.buf_unproc_raw.borrow();
                    let _t_buf_unproc_raw_io = BytesReader::from(buf_unproc_raw.clone());
                    let t = Self::read_into::<BytesReader, ProcessCoerceSwitch_Foo>(&_t_buf_unproc_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.buf_unproc.borrow_mut() = Some(t);
                }
                _ => {
                    *self_rc.buf_unproc.borrow_mut() = Some(_io.read_bytes_full()?.into());
                }
            }
        }
        if ((to_i128(*self_rc.flag())) != (to_i128(0))) {
            match *self_rc.buf_type() {
                0 => {
                    *self_rc.buf_proc_raw.borrow_mut() = _io.read_bytes(4_usize)?.into();
                    let buf_proc_raw = self_rc.buf_proc_raw.borrow();
                    let _t_buf_proc_raw_proc = process_xor_one(&buf_proc_raw, 170_u8);
                    let _t_buf_proc_raw_io = BytesReader::from(_t_buf_proc_raw_proc);
                    let t = Self::read_into::<BytesReader, ProcessCoerceSwitch_Foo>(&_t_buf_proc_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.buf_proc.borrow_mut() = Some(t);
                }
                _ => {
                    *self_rc.buf_proc.borrow_mut() = Some(process_xor_one(&_io.read_bytes_full()?, 170_u8).into());
                }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessCoerceSwitch {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn buf(
        &self
    ) -> KResult<Ref<'_, OptRc<ProcessCoerceSwitch_Foo>>> {
        let _io = self._io.borrow();
        if self.f_buf.get() {
            return Ok(self.buf.borrow());
        }
        *self.buf.borrow_mut() = if ((to_i128(*self.flag())) == (to_i128(0))) { self.buf_unproc() } else { self.buf_proc() }.clone();
        Ok(self.buf.borrow())
    }
}
impl ProcessCoerceSwitch {
    pub fn buf_type(&self) -> Ref<'_, u8> {
        self.buf_type.borrow()
    }
}
impl ProcessCoerceSwitch {
    pub fn flag(&self) -> Ref<'_, u8> {
        self.flag.borrow()
    }
}
impl ProcessCoerceSwitch {
    pub fn buf_unproc(&self) -> Ref<'_, Option<ProcessCoerceSwitch_BufUnproc>> {
        self.buf_unproc.borrow()
    }
}
impl ProcessCoerceSwitch {
    pub fn buf_proc(&self) -> Ref<'_, Option<ProcessCoerceSwitch_BufProc>> {
        self.buf_proc.borrow()
    }
}
impl ProcessCoerceSwitch {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessCoerceSwitch {
    pub fn buf_unproc_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf_unproc_raw.borrow()
    }
}
impl ProcessCoerceSwitch {
    pub fn buf_proc_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf_proc_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessCoerceSwitch_Foo {
    pub(crate) _root: SharedType<ProcessCoerceSwitch>,
    pub(crate) _parent: SharedType<ProcessCoerceSwitch>,
    pub(crate) _self_shared: SharedType<Self>,
    bar: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    bar_raw: RefCell<Vec<u8>>,
}
impl KStruct for ProcessCoerceSwitch_Foo {
    type Root = ProcessCoerceSwitch;
    type Parent = ProcessCoerceSwitch;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.bar.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessCoerceSwitch_Foo {
}
impl ProcessCoerceSwitch_Foo {
    pub fn bar(&self) -> Ref<'_, Vec<u8>> {
        self.bar.borrow()
    }
}
impl ProcessCoerceSwitch_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessCoerceSwitch_Foo {
    pub fn bar_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bar_raw.borrow()
    }
}
