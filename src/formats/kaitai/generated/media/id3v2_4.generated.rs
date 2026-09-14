// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <http://id3.org/id3v2.4.0-structure> Source
 * \sa <http://id3.org/id3v2.4.0-frames> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Id3v24 {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24>,
    pub(crate) _self_shared: SharedType<Self>,
    tag: RefCell<OptRc<Id3v24_Tag>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24 {
    type Root = Id3v24;
    type Parent = Id3v24;

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
        let t = Self::read_into::<_, Id3v24_Tag>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.tag.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24 {
}
impl Id3v24 {
    pub fn tag(&self) -> Ref<'_, OptRc<Id3v24_Tag>> {
        self.tag.borrow()
    }
}
impl Id3v24 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Footer {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Tag>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    version_major: RefCell<u8>,
    version_revision: RefCell<u8>,
    flags: RefCell<OptRc<Id3v24_Footer_Flags>>,
    size: RefCell<OptRc<Id3v24_U4beSynchsafe>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Footer {
    type Root = Id3v24;
    type Parent = Id3v24_Tag;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(3_usize)?;
        if !(*self_rc.magic() == vec![0x33u8, 0x44u8, 0x49u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/footer/seq/0".to_string() }));
        }
        *self_rc.version_major.borrow_mut() = _io.read_u1()?;
        *self_rc.version_revision.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, Id3v24_Footer_Flags>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_U4beSynchsafe>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.size.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Footer {
}
impl Id3v24_Footer {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Id3v24_Footer {
    pub fn version_major(&self) -> Ref<'_, u8> {
        self.version_major.borrow()
    }
}
impl Id3v24_Footer {
    pub fn version_revision(&self) -> Ref<'_, u8> {
        self.version_revision.borrow()
    }
}
impl Id3v24_Footer {
    pub fn flags(&self) -> Ref<'_, OptRc<Id3v24_Footer_Flags>> {
        self.flags.borrow()
    }
}
impl Id3v24_Footer {
    pub fn size(&self) -> Ref<'_, OptRc<Id3v24_U4beSynchsafe>> {
        self.size.borrow()
    }
}
impl Id3v24_Footer {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Footer_Flags {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Footer>,
    pub(crate) _self_shared: SharedType<Self>,
    flag_unsynchronization: RefCell<bool>,
    flag_headerex: RefCell<bool>,
    flag_experimental: RefCell<bool>,
    flag_footer: RefCell<bool>,
    reserved: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Footer_Flags {
    type Root = Id3v24;
    type Parent = Id3v24_Footer;

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
        *self_rc.flag_unsynchronization.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_headerex.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_experimental.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_footer.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Footer_Flags {
}
impl Id3v24_Footer_Flags {
    pub fn flag_unsynchronization(&self) -> Ref<'_, bool> {
        self.flag_unsynchronization.borrow()
    }
}
impl Id3v24_Footer_Flags {
    pub fn flag_headerex(&self) -> Ref<'_, bool> {
        self.flag_headerex.borrow()
    }
}
impl Id3v24_Footer_Flags {
    pub fn flag_experimental(&self) -> Ref<'_, bool> {
        self.flag_experimental.borrow()
    }
}
impl Id3v24_Footer_Flags {
    pub fn flag_footer(&self) -> Ref<'_, bool> {
        self.flag_footer.borrow()
    }
}
impl Id3v24_Footer_Flags {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}
impl Id3v24_Footer_Flags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Frame {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Tag>,
    pub(crate) _self_shared: SharedType<Self>,
    id: RefCell<String>,
    size: RefCell<OptRc<Id3v24_U4beSynchsafe>>,
    flags_status: RefCell<OptRc<Id3v24_Frame_FlagsStatus>>,
    flags_format: RefCell<OptRc<Id3v24_Frame_FlagsFormat>>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    id_raw: RefCell<Vec<u8>>,
    data_raw: RefCell<Vec<u8>>,
    f_is_invalid: Cell<bool>,
    is_invalid: RefCell<bool>,
}
impl KStruct for Id3v24_Frame {
    type Root = Id3v24;
    type Parent = Id3v24_Tag;

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
        *self_rc.id.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        let t = Self::read_into::<_, Id3v24_U4beSynchsafe>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.size.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_Frame_FlagsStatus>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags_status.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_Frame_FlagsFormat>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags_format.borrow_mut() = t;
        *self_rc.data.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.size().value()?)?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Frame {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_invalid(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_invalid.get() {
            return Ok(self.is_invalid.borrow());
        }
        self.f_is_invalid.set(true);
        *self.is_invalid.borrow_mut() = ((self.id().as_str() == "\0\0\0\0")).try_into()?;
        Ok(self.is_invalid.borrow())
    }
}
impl Id3v24_Frame {
    pub fn id(&self) -> Ref<'_, String> {
        self.id.borrow()
    }
}
impl Id3v24_Frame {
    pub fn size(&self) -> Ref<'_, OptRc<Id3v24_U4beSynchsafe>> {
        self.size.borrow()
    }
}
impl Id3v24_Frame {
    pub fn flags_status(&self) -> Ref<'_, OptRc<Id3v24_Frame_FlagsStatus>> {
        self.flags_status.borrow()
    }
}
impl Id3v24_Frame {
    pub fn flags_format(&self) -> Ref<'_, OptRc<Id3v24_Frame_FlagsFormat>> {
        self.flags_format.borrow()
    }
}
impl Id3v24_Frame {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Id3v24_Frame {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Id3v24_Frame {
    pub fn id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.id_raw.borrow()
    }
}
impl Id3v24_Frame {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Frame_FlagsFormat {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Frame>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<bool>,
    flag_grouping: RefCell<bool>,
    reserved2: RefCell<u64>,
    flag_compressed: RefCell<bool>,
    flag_encrypted: RefCell<bool>,
    flag_unsynchronisated: RefCell<bool>,
    flag_indicator: RefCell<bool>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Frame_FlagsFormat {
    type Root = Id3v24;
    type Parent = Id3v24_Frame;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_grouping.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved2.borrow_mut() = _io.read_bits_int_be(2)?;
        *self_rc.flag_compressed.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_encrypted.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_unsynchronisated.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_indicator.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Frame_FlagsFormat {
}
impl Id3v24_Frame_FlagsFormat {
    pub fn reserved1(&self) -> Ref<'_, bool> {
        self.reserved1.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn flag_grouping(&self) -> Ref<'_, bool> {
        self.flag_grouping.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn reserved2(&self) -> Ref<'_, u64> {
        self.reserved2.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn flag_compressed(&self) -> Ref<'_, bool> {
        self.flag_compressed.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn flag_encrypted(&self) -> Ref<'_, bool> {
        self.flag_encrypted.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn flag_unsynchronisated(&self) -> Ref<'_, bool> {
        self.flag_unsynchronisated.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn flag_indicator(&self) -> Ref<'_, bool> {
        self.flag_indicator.borrow()
    }
}
impl Id3v24_Frame_FlagsFormat {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Frame_FlagsStatus {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Frame>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<bool>,
    flag_discard_alter_tag: RefCell<bool>,
    flag_discard_alter_file: RefCell<bool>,
    flag_read_only: RefCell<bool>,
    reserved2: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Frame_FlagsStatus {
    type Root = Id3v24;
    type Parent = Id3v24_Frame;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_discard_alter_tag.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_discard_alter_file.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_read_only.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved2.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Frame_FlagsStatus {
}
impl Id3v24_Frame_FlagsStatus {
    pub fn reserved1(&self) -> Ref<'_, bool> {
        self.reserved1.borrow()
    }
}
impl Id3v24_Frame_FlagsStatus {
    pub fn flag_discard_alter_tag(&self) -> Ref<'_, bool> {
        self.flag_discard_alter_tag.borrow()
    }
}
impl Id3v24_Frame_FlagsStatus {
    pub fn flag_discard_alter_file(&self) -> Ref<'_, bool> {
        self.flag_discard_alter_file.borrow()
    }
}
impl Id3v24_Frame_FlagsStatus {
    pub fn flag_read_only(&self) -> Ref<'_, bool> {
        self.flag_read_only.borrow()
    }
}
impl Id3v24_Frame_FlagsStatus {
    pub fn reserved2(&self) -> Ref<'_, u64> {
        self.reserved2.borrow()
    }
}
impl Id3v24_Frame_FlagsStatus {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Header {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Tag>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    version_major: RefCell<u8>,
    version_revision: RefCell<u8>,
    flags: RefCell<OptRc<Id3v24_Header_Flags>>,
    size: RefCell<OptRc<Id3v24_U4beSynchsafe>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Header {
    type Root = Id3v24;
    type Parent = Id3v24_Tag;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(3_usize)?;
        if !(*self_rc.magic() == vec![0x49u8, 0x44u8, 0x33u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/0".to_string() }));
        }
        *self_rc.version_major.borrow_mut() = _io.read_u1()?;
        *self_rc.version_revision.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, Id3v24_Header_Flags>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_U4beSynchsafe>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.size.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Header {
}
impl Id3v24_Header {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Id3v24_Header {
    pub fn version_major(&self) -> Ref<'_, u8> {
        self.version_major.borrow()
    }
}
impl Id3v24_Header {
    pub fn version_revision(&self) -> Ref<'_, u8> {
        self.version_revision.borrow()
    }
}
impl Id3v24_Header {
    pub fn flags(&self) -> Ref<'_, OptRc<Id3v24_Header_Flags>> {
        self.flags.borrow()
    }
}
impl Id3v24_Header {
    pub fn size(&self) -> Ref<'_, OptRc<Id3v24_U4beSynchsafe>> {
        self.size.borrow()
    }
}
impl Id3v24_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Header_Flags {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    flag_unsynchronization: RefCell<bool>,
    flag_headerex: RefCell<bool>,
    flag_experimental: RefCell<bool>,
    flag_footer: RefCell<bool>,
    reserved: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Header_Flags {
    type Root = Id3v24;
    type Parent = Id3v24_Header;

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
        *self_rc.flag_unsynchronization.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_headerex.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_experimental.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_footer.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Header_Flags {
}
impl Id3v24_Header_Flags {
    pub fn flag_unsynchronization(&self) -> Ref<'_, bool> {
        self.flag_unsynchronization.borrow()
    }
}
impl Id3v24_Header_Flags {
    pub fn flag_headerex(&self) -> Ref<'_, bool> {
        self.flag_headerex.borrow()
    }
}
impl Id3v24_Header_Flags {
    pub fn flag_experimental(&self) -> Ref<'_, bool> {
        self.flag_experimental.borrow()
    }
}
impl Id3v24_Header_Flags {
    pub fn flag_footer(&self) -> Ref<'_, bool> {
        self.flag_footer.borrow()
    }
}
impl Id3v24_Header_Flags {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}
impl Id3v24_Header_Flags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_HeaderEx {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Tag>,
    pub(crate) _self_shared: SharedType<Self>,
    size: RefCell<OptRc<Id3v24_U4beSynchsafe>>,
    flags_ex: RefCell<OptRc<Id3v24_HeaderEx_FlagsEx>>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data_raw: RefCell<Vec<u8>>,
}
impl KStruct for Id3v24_HeaderEx {
    type Root = Id3v24;
    type Parent = Id3v24_Tag;

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
        let t = Self::read_into::<_, Id3v24_U4beSynchsafe>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.size.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_HeaderEx_FlagsEx>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags_ex.borrow_mut() = t;
        *self_rc.data.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc.size().value()?).saturating_sub(5_u64))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_HeaderEx {
}
impl Id3v24_HeaderEx {
    pub fn size(&self) -> Ref<'_, OptRc<Id3v24_U4beSynchsafe>> {
        self.size.borrow()
    }
}
impl Id3v24_HeaderEx {
    pub fn flags_ex(&self) -> Ref<'_, OptRc<Id3v24_HeaderEx_FlagsEx>> {
        self.flags_ex.borrow()
    }
}
impl Id3v24_HeaderEx {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Id3v24_HeaderEx {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Id3v24_HeaderEx {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_HeaderEx_FlagsEx {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_HeaderEx>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<bool>,
    flag_update: RefCell<bool>,
    flag_crc: RefCell<bool>,
    flag_restrictions: RefCell<bool>,
    reserved2: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_HeaderEx_FlagsEx {
    type Root = Id3v24;
    type Parent = Id3v24_HeaderEx;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_update.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_crc.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.flag_restrictions.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved2.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_HeaderEx_FlagsEx {
}
impl Id3v24_HeaderEx_FlagsEx {
    pub fn reserved1(&self) -> Ref<'_, bool> {
        self.reserved1.borrow()
    }
}
impl Id3v24_HeaderEx_FlagsEx {
    pub fn flag_update(&self) -> Ref<'_, bool> {
        self.flag_update.borrow()
    }
}
impl Id3v24_HeaderEx_FlagsEx {
    pub fn flag_crc(&self) -> Ref<'_, bool> {
        self.flag_crc.borrow()
    }
}
impl Id3v24_HeaderEx_FlagsEx {
    pub fn flag_restrictions(&self) -> Ref<'_, bool> {
        self.flag_restrictions.borrow()
    }
}
impl Id3v24_HeaderEx_FlagsEx {
    pub fn reserved2(&self) -> Ref<'_, u64> {
        self.reserved2.borrow()
    }
}
impl Id3v24_HeaderEx_FlagsEx {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Padding {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_Tag>,
    pub(crate) _self_shared: SharedType<Self>,
    padding: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    padding_raw: RefCell<Vec<u8>>,
}
impl KStruct for Id3v24_Padding {
    type Root = Id3v24;
    type Parent = Id3v24_Tag;

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
        *self_rc.padding.borrow_mut() = _io.read_bytes(usize::try_from((usize::try_from(*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.tag().header().size().value()?)?).saturating_sub(_io.pos()))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Padding {
}
impl Id3v24_Padding {
    pub fn padding(&self) -> Ref<'_, Vec<u8>> {
        self.padding.borrow()
    }
}
impl Id3v24_Padding {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Id3v24_Padding {
    pub fn padding_raw(&self) -> Ref<'_, Vec<u8>> {
        self.padding_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_Tag {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Id3v24_Header>>,
    header_ex: RefCell<OptRc<Id3v24_HeaderEx>>,
    frames: RefCell<Vec<OptRc<Id3v24_Frame>>>,
    padding: RefCell<OptRc<Id3v24_Padding>>,
    footer: RefCell<OptRc<Id3v24_Footer>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_Tag {
    type Root = Id3v24;
    type Parent = Id3v24;

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
        let t = Self::read_into::<_, Id3v24_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        if *self_rc.header().flags().flag_headerex() {
            let t = Self::read_into::<_, Id3v24_HeaderEx>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.header_ex.borrow_mut() = t;
        }
        *self_rc.frames.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, Id3v24_Frame>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.frames.borrow_mut().push(t);
                let _t_frames = self_rc.frames.borrow();
                let Some(_tmpa) = _t_frames.last() else { break; };
                _i = _i.saturating_add(1);
                if  ((((to_i128((_io.pos()).saturating_add(usize::try_from(*_tmpa.size().value()?)?))) > (to_i128(*self_rc.header().size().value()?)))) || (*_tmpa.is_invalid()?))  { break; }
            }
        }
        if !(*self_rc.header().flags().flag_footer()) {
            let t = Self::read_into::<_, Id3v24_Padding>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.padding.borrow_mut() = t;
        }
        if *self_rc.header().flags().flag_footer() {
            let t = Self::read_into::<_, Id3v24_Footer>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.footer.borrow_mut() = t;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_Tag {
}
impl Id3v24_Tag {
    pub fn header(&self) -> Ref<'_, OptRc<Id3v24_Header>> {
        self.header.borrow()
    }
}
impl Id3v24_Tag {
    pub fn header_ex(&self) -> Ref<'_, OptRc<Id3v24_HeaderEx>> {
        self.header_ex.borrow()
    }
}
impl Id3v24_Tag {
    pub fn frames(&self) -> Ref<'_, Vec<OptRc<Id3v24_Frame>>> {
        self.frames.borrow()
    }
}
impl Id3v24_Tag {
    pub fn padding(&self) -> Ref<'_, OptRc<Id3v24_Padding>> {
        self.padding.borrow()
    }
}
impl Id3v24_Tag {
    pub fn footer(&self) -> Ref<'_, OptRc<Id3v24_Footer>> {
        self.footer.borrow()
    }
}
impl Id3v24_Tag {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_U1beSynchsafe {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_U2beSynchsafe>,
    pub(crate) _self_shared: SharedType<Self>,
    padding: RefCell<bool>,
    value: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Id3v24_U1beSynchsafe {
    type Root = Id3v24;
    type Parent = Id3v24_U2beSynchsafe;

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
        *self_rc.padding.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.value.borrow_mut() = _io.read_bits_int_be(7)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_U1beSynchsafe {
}
impl Id3v24_U1beSynchsafe {
    pub fn padding(&self) -> Ref<'_, bool> {
        self.padding.borrow()
    }
}
impl Id3v24_U1beSynchsafe {
    pub fn value(&self) -> Ref<'_, u64> {
        self.value.borrow()
    }
}
impl Id3v24_U1beSynchsafe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_U2beSynchsafe {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<Id3v24_U4beSynchsafe>,
    pub(crate) _self_shared: SharedType<Self>,
    byte0: RefCell<OptRc<Id3v24_U1beSynchsafe>>,
    byte1: RefCell<OptRc<Id3v24_U1beSynchsafe>>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<u64>,
}
impl KStruct for Id3v24_U2beSynchsafe {
    type Root = Id3v24;
    type Parent = Id3v24_U4beSynchsafe;

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
        let t = Self::read_into::<_, Id3v24_U1beSynchsafe>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.byte0.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_U1beSynchsafe>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.byte1.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_U2beSynchsafe {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = (((u64::try_from((i32::try_from(*self.byte0().value())?).wrapping_shl(7_u32))?) | (*self.byte1().value()))).try_into()?;
        Ok(self.value.borrow())
    }
}
impl Id3v24_U2beSynchsafe {
    pub fn byte0(&self) -> Ref<'_, OptRc<Id3v24_U1beSynchsafe>> {
        self.byte0.borrow()
    }
}
impl Id3v24_U2beSynchsafe {
    pub fn byte1(&self) -> Ref<'_, OptRc<Id3v24_U1beSynchsafe>> {
        self.byte1.borrow()
    }
}
impl Id3v24_U2beSynchsafe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Id3v24_U4beSynchsafe {
    pub(crate) _root: SharedType<Id3v24>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    short0: RefCell<OptRc<Id3v24_U2beSynchsafe>>,
    short1: RefCell<OptRc<Id3v24_U2beSynchsafe>>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<u64>,
}
impl KStruct for Id3v24_U4beSynchsafe {
    type Root = Id3v24;
    type Parent = KStructUnit;

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
        let t = Self::read_into::<_, Id3v24_U2beSynchsafe>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.short0.borrow_mut() = t;
        let t = Self::read_into::<_, Id3v24_U2beSynchsafe>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.short1.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Id3v24_U4beSynchsafe {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = (((u64::try_from((i32::try_from(*self.short0().value()?)?).wrapping_shl(14_u32))?) | (*self.short1().value()?))).try_into()?;
        Ok(self.value.borrow())
    }
}
impl Id3v24_U4beSynchsafe {
    pub fn short0(&self) -> Ref<'_, OptRc<Id3v24_U2beSynchsafe>> {
        self.short0.borrow()
    }
}
impl Id3v24_U4beSynchsafe {
    pub fn short1(&self) -> Ref<'_, OptRc<Id3v24_U2beSynchsafe>> {
        self.short1.borrow()
    }
}
impl Id3v24_U4beSynchsafe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
