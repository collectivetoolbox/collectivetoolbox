// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * A variable-length unsigned integer using base128 encoding. 1-byte groups
 * consist of 1-bit flag of continuation and 7-bit value chunk, and are ordered
 * "most significant group first", i.e. in "big-endian" manner.
 *
 * This particular encoding is specified and used in:
 *
 * * Standard MIDI file format
 * * ASN.1 BER encoding
 * * RAR 5.0 file format
 *
 * More information on this encoding is available at
 * <https://en.wikipedia.org/wiki/Variable-length_quantity>
 *
 * This particular implementation supports serialized values to up 8 bytes long.
 */

#[derive(Default, Debug, Clone)]
pub struct VlqBase128Be {
    pub(crate) _root: SharedType<VlqBase128Be>,
    pub(crate) _parent: SharedType<VlqBase128Be>,
    pub(crate) _self_shared: SharedType<Self>,
    groups: RefCell<Vec<OptRc<VlqBase128Be_Group>>>,
    _io: RefCell<BytesReader>,
    f_last: Cell<bool>,
    last: RefCell<i32>,
    f_value: Cell<bool>,
    value: RefCell<i32>,
}
impl KStruct for VlqBase128Be {
    type Root = VlqBase128Be;
    type Parent = VlqBase128Be;

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
        *self_rc.groups.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, VlqBase128Be_Group>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.groups.borrow_mut().push(t);
                let _t_groups = self_rc.groups.borrow();
                let Some(_tmpa) = _t_groups.last() else { break; };
                _i = _i.saturating_add(1);
                if !(*_tmpa.has_next()) { break; }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl VlqBase128Be {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn last(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_last.get() {
            return Ok(self.last.borrow());
        }
        self.f_last.set(true);
        *self.last.borrow_mut() = ((self.groups().len()).saturating_sub(1_usize)).try_into()?;
        Ok(self.last.borrow())
    }

    /**
     * Resulting value as normal integer
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = ((((((((*self.groups().get(usize::try_from(*self.last()?)?).ok_or(KError::CastError)?.value()).saturating_add(u64::try_from(if *self.last()? >= 1 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(1_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(7_u32) } else { 0_i32 })?)).saturating_add(u64::try_from(if *self.last()? >= 2 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(2_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(14_u32) } else { 0_i32 })?)).saturating_add(u64::try_from(if *self.last()? >= 3 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(3_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(21_u32) } else { 0_i32 })?)).saturating_add(u64::try_from(if *self.last()? >= 4 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(4_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(28_u32) } else { 0_i32 })?)).saturating_add(u64::try_from(if *self.last()? >= 5 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(5_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(35_u32) } else { 0_i32 })?)).saturating_add(u64::try_from(if *self.last()? >= 6 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(6_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(42_u32) } else { 0_i32 })?)).saturating_add(u64::try_from(if *self.last()? >= 7 { (i32::try_from(*self.groups().get(usize::try_from((*self.last()?).saturating_sub(7_i32))?).ok_or(KError::CastError)?.value())?).wrapping_shl(49_u32) } else { 0_i32 })?)).try_into()?;
        Ok(self.value.borrow())
    }
}
impl VlqBase128Be {
    pub fn groups(&self) -> Ref<'_, Vec<OptRc<VlqBase128Be_Group>>> {
        self.groups.borrow()
    }
}
impl VlqBase128Be {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * One byte group, clearly divided into 7-bit "value" chunk and 1-bit "continuation" flag.
 */

#[derive(Default, Debug, Clone)]
pub struct VlqBase128Be_Group {
    pub(crate) _root: SharedType<VlqBase128Be>,
    pub(crate) _parent: SharedType<VlqBase128Be>,
    pub(crate) _self_shared: SharedType<Self>,
    has_next: RefCell<bool>,
    value: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for VlqBase128Be_Group {
    type Root = VlqBase128Be;
    type Parent = VlqBase128Be;

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
        *self_rc.has_next.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.value.borrow_mut() = _io.read_bits_int_be(7)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl VlqBase128Be_Group {
}

/**
 * If true, then we have more bytes to read
 */
impl VlqBase128Be_Group {
    pub fn has_next(&self) -> Ref<'_, bool> {
        self.has_next.borrow()
    }
}

/**
 * The 7-bit (base128) numeric value chunk of this group
 */
impl VlqBase128Be_Group {
    pub fn value(&self) -> Ref<'_, u64> {
        self.value.borrow()
    }
}
impl VlqBase128Be_Group {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
