// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from kaitai_struct_rust_runtime: MIT
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// License for parts derived from kaitai_struct_rust_runtime:

/*
MIT License

Copyright (c) 2019-2025 Kaitai Project

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

//! Kaitai Struct runtime support for parsing and stream operations.

#[allow(clippy::wildcard_imports, reason = "Standard workspace module prelude")]
pub(crate) use ctb_utilities::*;

use flate2::read::ZlibDecoder;

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt,
    io::{Read, Seek, SeekFrom},
    ops::Deref,
    path::Path,
    rc::{Rc, Weak},
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum KError {
    Eof { requested: usize, available: usize },
    EmptyIterator,
    UnknownEncoding { name: String },
    MissingRoot,
    MissingParent,
    ReadBitsTooLarge { requested: usize },
    ValidationFailed(ValidationFailedError),
    NoTerminatorFound,
    IoError { msg: String },
    BytesDecodingError { msg: String },
    CastError,
    UndecidedEndianness { src_path: String },
}
pub type KResult<T> = Result<T, KError>;

impl fmt::Display for KError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for KError {}

/// Details of the failed validation.
///
/// <div class="warning">
///
/// The content of this struct is likely to change in future Kaitai Struct versions.
///
/// </div>
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValidationFailedError {
    pub kind: ValidationKind,
    pub src_path: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum ValidationKind {
    NotEqual,
    LessThan,
    GreaterThan,
    NotAnyOf,
    NotInEnum,
    Expr,
}

pub trait CustomDecoder {
    fn decode(&self, bytes: &[u8]) -> Result<Vec<u8>, String>;
}

#[derive(Default)]
pub struct SharedType<T>(RefCell<Weak<T>>);

impl<T> Clone for SharedType<T> {
    fn clone(&self) -> Self {
        Self(RefCell::new(Weak::clone(&*self.0.borrow())))
    }
}

// stop recursion while printing
impl<T> fmt::Debug for SharedType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = &*self.0.borrow();
        match w.strong_count() {
            0 => write!(f, "SharedType(Empty)"),
            _ => write!(f, "SharedType(Weak({:?}))", Weak::<T>::as_ptr(w)),
        }
    }
}

impl<T> SharedType<T> {
    pub fn new(rc: Rc<T>) -> Self {
        Self(RefCell::new(Rc::downgrade(&rc)))
    }

    pub fn empty() -> Self {
        Self(RefCell::new(Weak::new()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().strong_count() == 0
    }

    pub fn get(&self) -> KResult<OptRc<T>> {
        match self.0.borrow().upgrade() {
            Some(rc) => Ok(OptRc::from(rc)),
            None => Err(KError::MissingParent),
        }
    }

    pub fn get_value(&self) -> &RefCell<Weak<T>> {
        &self.0
    }

    pub fn set(&self, rc: KResult<OptRc<T>>) {
        *self.0.borrow_mut() = match rc.ok() {
            Some(v) => Rc::downgrade(&v.get()),
            None => Weak::new(),
        }
    }
}

// we use own type OptRc<> instead of Rc<> only for one reason:
// by default to not create default value of type T (instead contain Option(None))
// (T could have cyclic-types inside, as a result we got stack overflow)
#[derive(Debug)]
pub struct OptRc<T>(Option<Rc<T>>);

impl<T> OptRc<T> {
    pub fn new(orc: &Option<Rc<T>>) -> Self {
        match orc {
            Some(rc) => OptRc::from(rc.clone()),
            None => OptRc::default(),
        }
    }

    #[expect(
        clippy::unwrap_used,
        reason = "OptRc acts as a smart pointer whose get() requires an initialized inner Rc"
    )]
    pub fn get(&self) -> Rc<T> {
        self.0.as_ref().unwrap().clone()
    }

    pub fn get_value(&self) -> &Option<Rc<T>> {
        &self.0
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn as_ref(&self) -> Option<&T> {
        self.0.as_deref()
    }

    #[expect(
        clippy::unwrap_used,
        reason = "OptRc acts as a smart pointer whose get_mut() requires an initialized inner Rc"
    )]
    pub fn get_mut(&mut self) -> &mut Rc<T> {
        self.0.as_mut().unwrap()
    }
}

impl<T> Default for OptRc<T> {
    #[inline]
    fn default() -> Self {
        OptRc(None)
    }
}

impl<T> Clone for OptRc<T> {
    fn clone(&self) -> Self {
        OptRc(self.0.clone())
    }
}

impl<T> From<Rc<T>> for OptRc<T> {
    fn from(v: Rc<T>) -> Self {
        OptRc(Some(v))
    }
}

impl<T> From<T> for OptRc<T> {
    fn from(v: T) -> Self {
        OptRc(Some(v.into()))
    }
}

impl<T> Deref for OptRc<T> {
    type Target = T;

    #[inline(always)]
    #[expect(
        clippy::unwrap_used,
        reason = "OptRc Deref contract mirrors pointer dereference requiring initialized inner value"
    )]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

pub trait KStruct: Default {
    type Root: KStruct;
    type Parent: KStruct;

    /// Parse this struct (and any children) from the supplied stream
    fn read<S: KStream>(
        self_rc: &OptRc<Self>,
        _io: &S,
        _root: SharedType<Self::Root>,
        _parent: SharedType<Self::Parent>,
    ) -> KResult<()>;

    /// helper function to read struct
    fn read_into<S: KStream, T: KStruct + Default + Any>(
        io: &S,
        root_in: Option<SharedType<T::Root>>,
        parent_in: Option<SharedType<T::Parent>>,
    ) -> KResult<OptRc<T>> {
        let t = OptRc::from(T::default());
        let root = Self::downcast(root_in, t.clone(), true);
        let parent = Self::downcast(parent_in, t.clone(), false);
        T::read(&t, io, root, parent)?;
        Ok(t)
    }

    /// helper function to special initialize and read struct
    fn read_into_with_init<S: KStream, T: KStruct + Default + Any>(
        io: &S,
        root_in: Option<SharedType<T::Root>>,
        parent_in: Option<SharedType<T::Parent>>,
        init: &dyn Fn(&mut T) -> KResult<()>,
    ) -> KResult<OptRc<T>> {
        let mut t = OptRc::from(T::default());
        if let Some(inner) = t.0.as_mut().and_then(Rc::get_mut) {
            init(inner)?;
        }

        let root = Self::downcast(root_in, t.clone(), true);
        let parent = Self::downcast(parent_in, t.clone(), false);
        T::read(&t, io, root, parent)?;
        Ok(t)
    }

    fn downcast<T, U>(opt_rc: Option<SharedType<U>>, t: OptRc<T>, _panic: bool) -> SharedType<U>
    where
        T: KStruct + Default + Any,
        U: 'static,
    {
        if let Some(rc) = opt_rc {
            rc
        } else {
            let t_any: &dyn Any = &t.get();
            match t_any.downcast_ref::<Rc<U>>() {
                Some(as_result) => SharedType::<U>::new(Rc::clone(as_result)),
                None => SharedType::<U>::empty(),
            }
        }
    }
}

/// Dummy struct used to indicate an absence of value; needed for
/// root structs to satisfy the associated type bounds in the
/// `KStruct` trait.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct KStructUnit;

impl KStruct for KStructUnit {
    type Root = KStructUnit;
    type Parent = KStructUnit;

    fn read<S: KStream>(
        _self_rc: &OptRc<Self>,
        _io: &S,
        _root: SharedType<Self::Root>,
        _parent: SharedType<Self::Parent>,
    ) -> KResult<()> {
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Struct {
    inner: Option<Rc<dyn std::any::Any>>,
}

impl std::fmt::Debug for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Struct({:?})", self.inner.is_some())
    }
}

impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        match (&self.inner, &other.inner) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Struct {
    pub fn new<T: 'static>(val: T) -> Self {
        Self {
            inner: Some(Rc::new(val)),
        }
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.inner.as_ref()?.downcast_ref::<T>()
    }
}

impl Struct {
    pub fn downcast_optrc<T: 'static + Clone>(&self) -> Result<OptRc<T>, KError> {
        self.inner
            .as_ref()
            .and_then(|a| {
                a.downcast_ref::<OptRc<T>>()
                    .cloned()
                    .or_else(|| a.downcast_ref::<T>().cloned().map(OptRc::from))
            })
            .ok_or(KError::CastError)
    }
}

impl OptRc<Struct> {
    pub fn downcast_optrc<T: 'static + Clone>(&self) -> Result<OptRc<T>, KError> {
        let inner_struct = self.as_ref().ok_or(KError::CastError)?;
        inner_struct.downcast_optrc()
    }
}

impl KStruct for Struct {
    type Root = Struct;
    type Parent = Struct;

    fn read<S: KStream>(
        _self_rc: &OptRc<Self>,
        _io: &S,
        _root: SharedType<Self::Root>,
        _parent: SharedType<Self::Parent>,
    ) -> KResult<()> {
        Ok(())
    }
}

pub type Io = BytesReader;

impl From<std::io::Error> for KError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            msg: err.to_string(),
        }
    }
}

impl From<std::num::TryFromIntError> for KError {
    fn from(_: std::num::TryFromIntError) -> Self {
        Self::CastError
    }
}

impl From<std::convert::Infallible> for KError {
    fn from(_: std::convert::Infallible) -> Self {
        Self::CastError
    }
}

fn to_fixed_array<const N: usize>(vec: Vec<u8>) -> KResult<[u8; N]> {
    vec.try_into().map_err(|v: Vec<u8>| KError::Eof {
        requested: N,
        available: v.len(),
    })
}

pub trait KStream {
    fn clone(&self) -> BytesReader;
    fn size(&self) -> usize;

    fn is_eof(&self) -> bool {
        if self.get_state().bits_left > 0 {
            return false;
        }
        self.pos() >= self.size()
    }

    fn seek(&self, position: usize) -> KResult<()> {
        self.align_to_byte()?;
        self.get_state_mut().pos = position;
        Ok(())
    }

    fn pos(&self) -> usize {
        self.get_state().pos
    }

    fn read_s1(&self) -> KResult<i8> {
        let bytes = self.read_bytes(1)?;
        let b = *bytes.first().ok_or(KError::Eof { requested: 1, available: 0 })?;
        Ok(i8::from_be_bytes([b]))
    }
    fn read_s2be(&self) -> KResult<i16> {
        Ok(i16::from_be_bytes(to_fixed_array(self.read_bytes(2)?)?))
    }
    fn read_s4be(&self) -> KResult<i32> {
        Ok(i32::from_be_bytes(to_fixed_array(self.read_bytes(4)?)?))
    }
    fn read_s8be(&self) -> KResult<i64> {
        Ok(i64::from_be_bytes(to_fixed_array(self.read_bytes(8)?)?))
    }
    fn read_s2le(&self) -> KResult<i16> {
        Ok(i16::from_le_bytes(to_fixed_array(self.read_bytes(2)?)?))
    }
    fn read_s4le(&self) -> KResult<i32> {
        Ok(i32::from_le_bytes(to_fixed_array(self.read_bytes(4)?)?))
    }
    fn read_s8le(&self) -> KResult<i64> {
        Ok(i64::from_le_bytes(to_fixed_array(self.read_bytes(8)?)?))
    }

    fn read_u1(&self) -> KResult<u8> {
        let bytes = self.read_bytes(1)?;
        let b = *bytes.first().ok_or(KError::Eof { requested: 1, available: 0 })?;
        Ok(b)
    }
    fn read_u2(&self) -> KResult<u16> {
        self.read_u2be()
    }
    fn read_u4(&self) -> KResult<u32> {
        self.read_u4be()
    }
    fn read_u2be(&self) -> KResult<u16> {
        Ok(u16::from_be_bytes(to_fixed_array(self.read_bytes(2)?)?))
    }
    fn read_u4be(&self) -> KResult<u32> {
        Ok(u32::from_be_bytes(to_fixed_array(self.read_bytes(4)?)?))
    }
    fn read_u8be(&self) -> KResult<u64> {
        Ok(u64::from_be_bytes(to_fixed_array(self.read_bytes(8)?)?))
    }
    fn read_u2le(&self) -> KResult<u16> {
        Ok(u16::from_le_bytes(to_fixed_array(self.read_bytes(2)?)?))
    }
    fn read_u4le(&self) -> KResult<u32> {
        Ok(u32::from_le_bytes(to_fixed_array(self.read_bytes(4)?)?))
    }
    fn read_u8le(&self) -> KResult<u64> {
        Ok(u64::from_le_bytes(to_fixed_array(self.read_bytes(8)?)?))
    }

    fn read_f4be(&self) -> KResult<f32> {
        Ok(f32::from_be_bytes(to_fixed_array(self.read_bytes(4)?)?))
    }
    fn read_f8be(&self) -> KResult<f64> {
        Ok(f64::from_be_bytes(to_fixed_array(self.read_bytes(8)?)?))
    }
    fn read_f4le(&self) -> KResult<f32> {
        Ok(f32::from_le_bytes(to_fixed_array(self.read_bytes(4)?)?))
    }
    fn read_f8le(&self) -> KResult<f64> {
        Ok(f64::from_le_bytes(to_fixed_array(self.read_bytes(8)?)?))
    }

    fn get_state(&self) -> Ref<'_, ReaderState>;
    fn get_state_mut(&self) -> RefMut<'_, ReaderState>;

    fn align_to_byte(&self) -> KResult<()> {
        let mut inner = self.get_state_mut();
        inner.bits = 0;
        inner.bits_left = 0;
        Ok(())
    }

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Bit manipulation algorithms in Kaitai runtime require bitwise shifts and arithmetic within guarded bounds (n <= 64)"
    )]
    fn read_bits_int_be(&self, n: usize) -> KResult<u64> {
        let mut res: u64 = 0;

        if n > 64 {
            return Err(KError::ReadBitsTooLarge { requested: n });
        }

        let n_i32: i32 = i32::try_from(n).map_err(|_err| KError::ReadBitsTooLarge { requested: n })?;
        let bits_needed = n_i32 - self.get_state().bits_left;
        self.get_state_mut().bits_left = -bits_needed & 7;

        if bits_needed > 0 {
            let bytes_needed = ((bits_needed - 1) / 8) + 1;
            let bytes_len = usize::try_from(bytes_needed).map_err(|_err| KError::ReadBitsTooLarge { requested: n })?;
            let buf = self.read_bytes_not_aligned(bytes_len)?;
            for b in buf {
                res = res << 8 | u64::from(b);
            }
            let mut inner = self.get_state_mut();
            let new_bits = res;
            res >>= inner.bits_left;
            if bits_needed < 64 {
                res |= inner.bits << bits_needed;
            }
            inner.bits = new_bits;
        } else {
            res = self.get_state().bits >> -bits_needed;
        }

        let mut inner = self.get_state_mut();
        let mask = (1u64 << inner.bits_left) - 1;
        inner.bits &= mask;

        Ok(res)
    }

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Bit manipulation algorithms in Kaitai runtime require bitwise shifts and arithmetic within guarded bounds (n <= 64)"
    )]
    fn read_bits_int_le(&self, n: usize) -> KResult<u64> {
        let mut res: u64 = 0;

        if n > 64 {
            return Err(KError::ReadBitsTooLarge { requested: n });
        }

        let n_i32: i32 = i32::try_from(n).map_err(|_err| KError::ReadBitsTooLarge { requested: n })?;
        let bits_needed = n_i32 - self.get_state().bits_left;

        if bits_needed > 0 {
            let bytes_needed = ((bits_needed - 1) / 8) + 1;
            let bytes_len = usize::try_from(bytes_needed).map_err(|_err| KError::ReadBitsTooLarge { requested: n })?;
            let buf = self.read_bytes_not_aligned(bytes_len)?;
            for (i, &b) in buf.iter().enumerate() {
                res |= u64::from(b) << (i * 8);
            }
            let mut inner = self.get_state_mut();
            let new_bits = if bits_needed < 64 {
                res >> bits_needed
            } else {
                0
            };
            res = res << inner.bits_left | inner.bits;
            inner.bits = new_bits;
        } else {
            let mut inner = self.get_state_mut();
            res = inner.bits;
            inner.bits >>= n;
        }

        self.get_state_mut().bits_left = -bits_needed & 7;

        if n < 64 {
            let mask = (1u64 << n) - 1;
            res &= mask;
        }

        Ok(res)
    }

    fn read_bytes(&self, len: usize) -> KResult<Vec<u8>> {
        self.align_to_byte()?;
        self.read_bytes_not_aligned(len)
    }

    fn read_bytes_not_aligned(&self, len: usize) -> KResult<Vec<u8>>;

    fn read_bytes_full(&self) -> KResult<Vec<u8>>;

    fn read_bytes_term(
        &self,
        term: u8,
        include: bool,
        consume: bool,
        eos_error: bool,
    ) -> KResult<Vec<u8>> {
        self.align_to_byte()?;
        let mut buf = vec![];
        loop {
            let c = match self.read_u1() {
                Ok(c) => c,
                Err(KError::Eof { .. }) => {
                    if eos_error {
                        return Err(KError::NoTerminatorFound);
                    }
                    return Ok(buf);
                }
                Err(e) => return Err(e),
            };
            if c == term {
                if include {
                    buf.push(c);
                }
                if !consume {
                    let mut state = self.get_state_mut();
                    state.pos = state.pos.saturating_sub(1);
                }
                return Ok(buf);
            }
            buf.push(c);
        }
    }

    fn read_bytes_term_multi(
        &self,
        term: &[u8],
        include: bool,
        consume: bool,
        eos_error: bool,
    ) -> KResult<Vec<u8>> {
        self.align_to_byte()?;
        let unit_len = term.len();
        if let Some(&term_byte) = term.first() {
            if unit_len == 1 {
                return self.read_bytes_term(term_byte, include, consume, eos_error);
            }
        } else {
            return self.read_bytes_full();
        }
        let mut buf = vec![];
        loop {
            let unit = match self.read_bytes(unit_len) {
                Ok(u) => u,
                Err(KError::Eof { .. }) => {
                    if eos_error {
                        return Err(KError::NoTerminatorFound);
                    }
                    return Ok(buf);
                }
                Err(e) => return Err(e),
            };
            if unit.as_slice() == term {
                if include {
                    buf.extend_from_slice(&unit);
                }
                if !consume {
                    let mut state = self.get_state_mut();
                    state.pos = state.pos.saturating_sub(unit_len);
                }
                return Ok(buf);
            }
            buf.extend_from_slice(&unit);
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ReaderState {
    pos: usize,
    bits: u64,
    bits_left: i32,
}

trait ReadSeek: Read + Seek {}

impl<T> ReadSeek for T where T: Read + Seek {}

impl fmt::Display for dyn ReadSeek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReadSeek")
    }
}

impl fmt::Debug for dyn ReadSeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReadSeek")
    }
}

#[derive(Debug, Default, Clone)]
pub struct BytesReader {
    state: RefCell<ReaderState>,
    // share same "instance" of data beetween all clones
    // reposition before each read call
    buf: OptRc<RefCell<Box<dyn ReadSeek>>>,
    file_size: u64,
}

impl From<Vec<u8>> for BytesReader {
    fn from(bytes: Vec<u8>) -> BytesReader {
        BytesReader::from_buffer(bytes)
    }
}

impl From<&[u8]> for BytesReader {
    fn from(slice: &[u8]) -> BytesReader {
        BytesReader::from_buffer(slice.to_vec())
    }
}

impl BytesReader {
    pub fn open<T: AsRef<Path>>(filename: T) -> KResult<Self> {
        let f = std::fs::File::open(filename)?;
        let file_size = f.metadata()?.len();
        let r: Box<dyn ReadSeek> = Box::new(f);
        Ok(BytesReader {
            state: RefCell::new(ReaderState::default()),
            file_size,
            buf: OptRc::from(RefCell::new(r)),
        })
    }

    fn from_buffer(bytes: Vec<u8>) -> Self {
        // Reason for fallback: in-memory buffer length exceeding u64 defaults to empty stream size 0
        let file_size = u64::try_from(bytes.len()).unwrap_or(0);
        let r: Box<dyn ReadSeek> = Box::new(std::io::Cursor::new(bytes));
        BytesReader {
            state: RefCell::new(ReaderState::default()),
            file_size,
            buf: OptRc::from(RefCell::new(r)),
        }
    }

    // sync stream pos with state.pos
    fn sync_pos(&self) -> KResult<()> {
        let cur_pos = self
            .buf
            .borrow_mut()
            .stream_position()?;
        // Reason for fallback: seek position exceeding target platform pointer width defaults to 0 offset
        let cur_pos_usize = usize::try_from(cur_pos).unwrap_or(0);
        if self.pos() != cur_pos_usize {
            // Reason for fallback: stream position exceeding u64 seek range defaults to beginning of stream
            let pos_u64 = u64::try_from(self.pos()).unwrap_or(0);
            self.buf
                .borrow_mut()
                .seek(SeekFrom::Start(pos_u64))?;
        }
        Ok(())
    }
}

impl KStream for BytesReader {
    fn clone(&self) -> Self {
        Clone::clone(self)
    }

    fn get_state(&self) -> Ref<'_, ReaderState> {
        self.state.borrow()
    }

    fn get_state_mut(&self) -> RefMut<'_, ReaderState> {
        self.state.borrow_mut()
    }

    fn size(&self) -> usize {
        // Reason for fallback: file size exceeding pointer width cannot be represented in usize and defaults to 0
        usize::try_from(self.file_size).unwrap_or(0)
    }

    fn read_bytes_not_aligned(&self, len: usize) -> KResult<Vec<u8>> {
        // handle read beyond end of file
        let num_bytes_available = self.size().saturating_sub(self.pos());
        if len > num_bytes_available {
            return Err(KError::Eof {
                requested: len,
                available: num_bytes_available,
            });
        }
        self.sync_pos()?;
        // let state = self.state.borrow_mut();
        // state.buf.resize(len, 0);
        let mut buf = vec![0; len];
        self
            .buf
            .borrow_mut()
            .read_exact(&mut buf[..])?;
        let mut state = self.get_state_mut();
        state.pos = state.pos.saturating_add(len);
        Ok(buf)
    }

    fn read_bytes_full(&self) -> KResult<Vec<u8>> {
        self.align_to_byte()?;
        self.sync_pos()?;
        //let state = self.state.borrow_mut();
        let mut buf = Vec::new();
        let readed = self
            .buf
            .borrow_mut()
            .read_to_end(&mut buf)?;
        let mut state = self.get_state_mut();
        state.pos = state.pos.saturating_add(readed);
        Ok(buf)
    }
}

/// Return a byte array that is sized to exclude all trailing instances of the
/// padding character.
pub fn bytes_strip_right(bytes: &[u8], pad: u8) -> Vec<u8> {
    if let Some(last_non_pad_index) = bytes.iter().rposition(|&c| c != pad) {
        #[expect(
            clippy::expect_used,
            reason = "last_non_pad_index returned by rposition is provably a valid index in bytes"
        )]
        bytes.get(..=last_non_pad_index).expect("valid index").to_vec()
    } else {
        vec![]
    }
}

/// Return a byte array that contains all bytes up until the
/// termination byte. Can optionally include the termination byte as well.
pub fn bytes_terminate(bytes: &[u8], term: u8, include_term: bool) -> Vec<u8> {
    if let Some(term_index) = bytes.iter().position(|&c| c == term) {
        let end = if include_term {
            term_index.saturating_add(1)
        } else {
            term_index
        };
        #[expect(
            clippy::expect_used,
            reason = "end is bounded by position in bytes plus at most 1"
        )]
        bytes.get(..end).expect("valid range").to_vec()
    } else {
        bytes.to_vec()
    }
}

/// Return a byte array that contains all bytes up until the multi-byte
/// termination sequence. Can optionally include the termination sequence as well.
pub fn bytes_terminate_multi(bytes: &[u8], term: &[u8], include_term: bool) -> Vec<u8> {
    let unit_len = term.len();
    if let Some(&term_byte) = term.first() {
        if unit_len == 1 {
            return bytes_terminate(bytes, term_byte, include_term);
        }
    } else {
        return bytes.to_vec();
    }
    let mut i = 0_usize;
    while i.saturating_add(unit_len) <= bytes.len() {
        if let Some(chunk) = bytes.get(i..i.saturating_add(unit_len)) {
            if chunk == term {
                let end = if include_term {
                    i.saturating_add(unit_len)
                } else {
                    i
                };
                if let Some(sub) = bytes.get(..end) {
                    return sub.to_vec();
                }
                return bytes.to_vec();
            }
        }
        i = i.saturating_add(unit_len);
    }
    bytes.to_vec()
}

/// Return a byte array terminating at `term` if present (taking precedence),
/// or stripping `pad_right` if terminator is not found (or not specified).
pub fn bytes_terminate_pad(
    bytes: &[u8],
    term: Option<u8>,
    include_term: bool,
    pad: Option<u8>,
) -> Vec<u8> {
    if let Some(t) = term {
        if let Some(pos) = bytes.iter().position(|&b| b == t) {
            let end = if include_term {
                pos.saturating_add(1)
            } else {
                pos
            };
            if let Some(sub) = bytes.get(..end) {
                return sub.to_vec();
            }
            return bytes.to_vec();
        }
    }
    if let Some(p) = pad {
        bytes_strip_right(bytes, p)
    } else {
        bytes.to_vec()
    }
}

/// Return a byte array terminating at multi-byte `term` if present (taking precedence),
/// or stripping `pad_right` if terminator is not found (or not specified).
pub fn bytes_terminate_pad_multi(
    bytes: &[u8],
    term: Option<&[u8]>,
    include_term: bool,
    pad: Option<u8>,
) -> Vec<u8> {
    if let Some(t) = term {
        let unit_len = t.len();
        if let Some(&t_byte) = t.first() {
            if unit_len == 1 {
                return bytes_terminate_pad(bytes, Some(t_byte), include_term, pad);
            }
        } else {
            return bytes.to_vec();
        }
        let mut i = 0_usize;
        while i.saturating_add(unit_len) <= bytes.len() {
            if let Some(chunk) = bytes.get(i..i.saturating_add(unit_len)) {
                if chunk == t {
                    let end = if include_term {
                        i.saturating_add(unit_len)
                    } else {
                        i
                    };
                    if let Some(sub) = bytes.get(..end) {
                        return sub.to_vec();
                    }
                    return bytes.to_vec();
                }
            }
            i = i.saturating_add(unit_len);
        }
    }
    if let Some(p) = pad {
        bytes_strip_right(bytes, p)
    } else {
        bytes.to_vec()
    }
}

pub fn bytes_to_str(bytes: &[u8], label: &str) -> KResult<String> {
    if label.eq_ignore_ascii_case("cp437") || label.eq_ignore_ascii_case("ibm437") {
        return ctb_formats_encoding::decode(
            ctb_formats_encoding::CharEncoding::cp437(),
            bytes,
        )
        .map_err(|e| KError::BytesDecodingError {
            msg: e.to_string(),
        });
    }

    if let Some(enc) = encoding_rs::Encoding::for_label(label.as_bytes()) {
        let (cow, _had_errors) = enc.decode_without_bom_handling(bytes);
        return Ok(cow.into_owned());
    }

    Err(KError::UnknownEncoding {
        name: label.to_string(),
    })
}

pub fn process_xor_one(bytes: &[u8], key: u8) -> Vec<u8> {
    let mut res = bytes.to_vec();
    for i in &mut res {
        *i ^= key;
    }
    res
}

pub fn process_xor_many(bytes: &[u8], key: &[u8]) -> Vec<u8> {
    let mut res = bytes.to_vec();
    if key.is_empty() {
        return res;
    }
    let mut ki = 0;
    for i in &mut res {
        if let Some(k) = key.get(ki) {
            *i ^= *k;
        }
        ki = ki.saturating_add(1);
        if ki >= key.len() {
            ki = 0;
        }
    }
    res
}

pub fn process_rotate_left(bytes: &[u8], amount: i64) -> Vec<u8> {
    let mut res = bytes.to_vec();
    // Reason for fallback: rem_euclid(8) is always in 0..8, so try_from into u32 is infallible
    let shift = u32::try_from(amount.rem_euclid(8)).unwrap_or(0);
    for i in &mut res {
        *i = i.rotate_left(shift);
    }
    res
}

pub fn process_rotate_right(bytes: &[u8], amount: i64) -> Vec<u8> {
    let mut res = bytes.to_vec();
    // Reason for fallback: rem_euclid(8) is always in 0..8, so try_from into u32 is infallible
    let shift = u32::try_from(amount.rem_euclid(8)).unwrap_or(0);
    for i in &mut res {
        *i = i.rotate_right(shift);
    }
    res
}

pub fn process_zlib(bytes: &[u8]) -> KResult<Vec<u8>> {
    let mut dec = ZlibDecoder::new(bytes);
    let mut dec_bytes = Vec::new();
    dec.read_to_end(&mut dec_bytes)
        .map_err(|e| KError::BytesDecodingError { msg: e.to_string() })?;
    Ok(dec_bytes)
}

pub fn reverse_string<S: AsRef<str>>(s: S) -> KResult<String> {
    Ok(s.as_ref().graphemes(true).rev().collect())
}

pub fn modulo(a: i64, b: i64) -> i64 {
    a.rem_euclid(b)
}

/// Performs integer division with floor rounding (towards negative infinity),
/// matching Kaitai Struct language specification semantics.
pub fn div_floor(a: i64, b: i64) -> KResult<i64> {
    if b == 0 {
        return Err(KError::CastError);
    }
    let d = a.checked_div(b).ok_or(KError::CastError)?;
    let r = a.checked_rem(b).ok_or(KError::CastError)?;
    if (r > 0 && b < 0) || (r < 0 && b > 0) {
        Ok(d.saturating_sub(1))
    } else {
        Ok(d)
    }
}

/// Extracts a substring slice safely without panicking on out-of-bounds or invalid UTF-8 boundaries.
pub fn substring<I1: TryInto<usize>, I2: TryInto<usize>>(s: &str, from: I1, to: I2) -> &str {
    let Ok(from) = from.try_into() else { return ""; };
    let Ok(to) = to.try_into() else { return ""; };
    // Reason for fallback: out of bounds or invalid UTF-8 boundary substring defaults to empty string
    s.get(from..to).unwrap_or("")
}

/// Converts an integer of any width safely to i128 for comparisons.
pub fn to_i128<T: TryInto<i128>>(val: T) -> i128 {
    // Reason for fallback: out of range integer for comparison defaults to 0
    val.try_into().unwrap_or(0)
}

/// Converts an integer of any width safely to i64.
pub fn to_i64<T: TryInto<i64>>(val: T) -> i64 {
    // Reason for fallback: out of range integer defaults to 0
    val.try_into().unwrap_or(0)
}

/// Converts an integer of any width safely to i32.
pub fn to_i32<T: TryInto<i32>>(val: T) -> i32 {
    // Reason for fallback: out of range integer defaults to 0
    val.try_into().unwrap_or(0)
}

/// Safely converts a shift amount to u32 for bitwise shift operations.
pub fn to_shift_amt<T: TryInto<u32>>(val: T) -> u32 {
    // Reason for fallback: out of range bitwise shift amount defaults to 0
    val.try_into().unwrap_or(0)
}

/// Converts a floating-point number to an integer for Kaitai expressions (truncates toward zero).
pub fn float_to_int<F: Into<f64>>(f: F) -> KResult<i64> {
    let val: f64 = f.into();
    utilities::math::exact_float::f64_to_i64(val.trunc())
        .map_err(|_| KError::CastError)
}

/// Trait for converting primitive numeric types to f64 for Kaitai expressions.
pub trait ToF64 {
    /// Converts `self` to `f64`.
    fn to_f64(self) -> f64;
}

/// Trait for converting primitive numeric types to f32 for Kaitai expressions.
pub trait ToF32 {
    /// Converts `self` to `f32`.
    fn to_f32(self) -> f32;
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

impl ToF64 for f32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}

impl ToF32 for f32 {
    fn to_f32(self) -> f32 {
        self
    }
}

impl ToF32 for f64 {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::f64_to_f32_approx(self).unwrap_or(0.0)
    }
}

impl ToF64 for u8 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for u16 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for u32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for i8 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for i16 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for i32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for u64 {
    fn to_f64(self) -> f64 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::u64_to_f64_approx(self).unwrap_or(0.0)
    }
}
impl ToF64 for usize {
    fn to_f64(self) -> f64 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::usize_to_f64_approx(self).unwrap_or(0.0)
    }
}
impl ToF64 for i64 {
    fn to_f64(self) -> f64 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::i64_to_f64_approx(self).unwrap_or(0.0)
    }
}
impl ToF64 for isize {
    fn to_f64(self) -> f64 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::isize_to_f64_approx(self).unwrap_or(0.0)
    }
}

impl ToF32 for u8 {
    fn to_f32(self) -> f32 {
        f32::from(self)
    }
}
impl ToF32 for u16 {
    fn to_f32(self) -> f32 {
        f32::from(self)
    }
}
impl ToF32 for i8 {
    fn to_f32(self) -> f32 {
        f32::from(self)
    }
}
impl ToF32 for i16 {
    fn to_f32(self) -> f32 {
        f32::from(self)
    }
}
impl ToF32 for u32 {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::u32_to_f32_approx(self).unwrap_or(0.0)
    }
}
impl ToF32 for u64 {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::u64_to_f32_approx(self).unwrap_or(0.0)
    }
}
impl ToF32 for usize {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::usize_to_f32_approx(self).unwrap_or(0.0)
    }
}
impl ToF32 for i32 {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::i32_to_f32_approx(self).unwrap_or(0.0)
    }
}
impl ToF32 for i64 {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::i64_to_f32_approx(self).unwrap_or(0.0)
    }
}
impl ToF32 for isize {
    fn to_f32(self) -> f32 {
        // Reason for fallback: out-of-range numeric value in float conversion defaults to 0.0
        utilities::math::approx_float::isize_to_f32_approx(self).unwrap_or(0.0)
    }
}

/// Converts a numeric value safely to f64 for floating-point calculations.
pub fn to_f64<T: ToF64>(val: T) -> f64 {
    val.to_f64()
}

/// Converts a numeric value safely to f32 for floating-point calculations.
pub fn to_f32<T: ToF32>(val: T) -> f32 {
    val.to_f32()
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn basic_strip_right() {
        let b = vec![1, 2, 3, 4, 5, 5, 5, 5];
        let c = bytes_strip_right(&b, 5);

        assert_eq!([1, 2, 3, 4], c[..]);
    }

    #[crate::ctb_test]
    fn basic_read_bytes() {
        let b = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let reader = BytesReader::from(b);

        assert_eq!(reader.read_bytes(4).unwrap()[..], [1, 2, 3, 4]);
        assert_eq!(reader.read_bytes(3).unwrap()[..], [5, 6, 7]);
        assert_eq!(
            reader.read_bytes(4).unwrap_err(),
            KError::Eof {
                requested: 4,
                available: 1
            }
        );
        assert_eq!(reader.read_bytes(1).unwrap()[..], [8]);
    }

    #[crate::ctb_test]
    fn read_bits_single() {
        let b = vec![0x80];
        let reader = BytesReader::from(b);

        assert_eq!(reader.read_bits_int_be(1).unwrap(), 1);
    }

    #[crate::ctb_test]
    fn read_bits_multiple() {
        // 0xA0
        let b = vec![0b10100000];
        let reader = BytesReader::from(b);

        assert_eq!(reader.read_bits_int_be(1).unwrap(), 1);
        assert_eq!(reader.read_bits_int_be(1).unwrap(), 0);
        assert_eq!(reader.read_bits_int_be(1).unwrap(), 1);
    }

    #[crate::ctb_test]
    fn read_bits_large() {
        let b = vec![0b10100000];
        let reader = BytesReader::from(b);

        assert_eq!(reader.read_bits_int_be(3).unwrap(), 5);
    }

    #[crate::ctb_test]
    fn read_bits_span() {
        let b = vec![0x01, 0x80];
        let reader = BytesReader::from(b);

        assert_eq!(reader.read_bits_int_be(9).unwrap(), 3);
    }

    #[crate::ctb_test]
    fn read_bits_too_large() {
        let b: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let reader = BytesReader::from(b);

        assert_eq!(
            reader.read_bits_int_be(65).unwrap_err(),
            KError::ReadBitsTooLarge { requested: 65 }
        )
    }

    #[crate::ctb_test]
    fn read_bytes_term() {
        let b = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let reader = BytesReader::from(b);

        assert_eq!(
            reader.read_bytes_term(3, false, false, false).unwrap()[..],
            [1, 2]
        );
        assert_eq!(
            reader.read_bytes_term(3, true, false, true).unwrap()[..],
            [3]
        );
        assert!(
            reader.read_bytes_term(3, false, true, true).unwrap().is_empty()
        );
        assert_eq!(
            reader.read_bytes_term(5, true, true, true).unwrap()[..],
            [4, 5]
        );
        assert_eq!(
            reader.read_bytes_term(8, false, false, true).unwrap()[..],
            [6, 7]
        );
        assert_eq!(
            reader.read_bytes_term(11, false, true, true).unwrap_err(),
            KError::NoTerminatorFound
        );
        // restore position
        reader.seek(7).unwrap();
        assert_eq!(
            reader.read_bytes_term(9, true, true, false).unwrap()[..],
            [8, 9]
        );
        assert_eq!(
            reader.read_bytes_term(10, true, false, false).unwrap()[..],
            [10]
        );
    }

    #[crate::ctb_test]
    fn process_xor_one_test() {
        let b = vec![0x66];
        let reader = BytesReader::from(b);
        let res = process_xor_one(&reader.read_bytes(1).unwrap(), 3);
        assert_eq!(0x65, res[0]);
    }

    #[crate::ctb_test]
    fn process_xor_many_test() {
        let b = vec![0x66, 0x6F];
        let reader = BytesReader::from(b);
        let key: Vec<u8> = vec![3, 3];
        let res = process_xor_many(&reader.read_bytes(2).unwrap(), &key);
        assert_eq!(vec![0x65, 0x6C], res);
    }

    #[crate::ctb_test]
    fn process_rotate_left_test() {
        let b = vec![0x09, 0xAC];
        let reader = BytesReader::from(b);
        let res = process_rotate_left(&reader.read_bytes(2).unwrap(), 3);
        let expected: Vec<u8> = vec![0x48, 0x65];
        assert_eq!(expected, res);
    }

    #[crate::ctb_test]
    fn basic_seek() {
        let b = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let reader = BytesReader::from(b);

        assert_eq!(reader.read_bytes(4).unwrap()[..], [1, 2, 3, 4]);
        let pos = reader.pos();
        reader.seek(1).unwrap();
        assert_eq!(reader.read_bytes(4).unwrap()[..], [2, 3, 4, 5]);
        reader.seek(pos).unwrap();
        assert_eq!(reader.read_bytes(4).unwrap()[..], [5, 6, 7, 8]);
        reader.seek(9).unwrap();
    }

    #[crate::ctb_test]
    fn test_bytes_to_str_cp437() -> anyhow::Result<()> {
        let bytes = vec![0x30, 0x31, 0x41, 0x42, 0xdb, 0x9b];
        let s = bytes_to_str(&bytes, "cp437")
            .map_err(|e| anyhow::anyhow!("Decoding failed: {e:?}"))?;
        assert_eq!(s, "01AB█¢");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_bytes_to_str_utf8() -> anyhow::Result<()> {
        let bytes = "Hello, 世界!".as_bytes().to_vec();
        let s = bytes_to_str(&bytes, "utf-8")
            .map_err(|e| anyhow::anyhow!("Decoding failed: {e:?}"))?;
        assert_eq!(s, "Hello, 世界!");
        Ok(())
    }

    fn dump_and_open(bytes: &[u8]) -> (tempfile::TempDir, BytesReader) {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("test.txt");
        std::fs::write(&file_path, bytes).unwrap();
        let reader = BytesReader::open(file_path).unwrap();
        (tmp_dir, reader)
    }

    #[crate::ctb_test]
    fn basic_read_bytes_file() {
        let (_tmp, reader) = dump_and_open(&[1, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(reader.read_bytes(4).unwrap()[..], [1, 2, 3, 4]);
        assert_eq!(reader.read_bytes(3).unwrap()[..], [5, 6, 7]);
        assert_eq!(
            reader.read_bytes(4).unwrap_err(),
            KError::Eof {
                requested: 4,
                available: 1
            }
        );
        assert_eq!(reader.read_bytes(1).unwrap()[..], [8]);
    }

    #[crate::ctb_test]
    fn basic_seek_file() {
        let (_tmp, reader) = dump_and_open(&[1, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(reader.read_bytes(4).unwrap()[..], [1, 2, 3, 4]);
        let pos = reader.pos();
        reader.seek(1).unwrap();
        assert_eq!(reader.read_bytes(4).unwrap()[..], [2, 3, 4, 5]);
        reader.seek(pos).unwrap();
        assert_eq!(reader.read_bytes(4).unwrap()[..], [5, 6, 7, 8]);
        reader.seek(9).unwrap();
    }
}
