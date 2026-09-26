// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Mozilla ARchive file is Mozilla's own archive format to distribute software updates.
 * Test files can be found on Mozilla's FTP site, for example:
 *
 * <http://ftp.mozilla.org/pub/firefox/nightly/partials/>
 * \sa <https://wiki.mozilla.org/Software_Update:MAR> Source
 */

#[derive(Default, Debug, Clone)]
pub struct MozillaMar {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    ofs_index: RefCell<u32>,
    file_size: RefCell<u64>,
    len_signatures: RefCell<u32>,
    signatures: RefCell<Vec<OptRc<MozillaMar_Signature>>>,
    len_additional_sections: RefCell<u32>,
    additional_sections: RefCell<Vec<OptRc<MozillaMar_AdditionalSection>>>,
    _io: RefCell<BytesReader>,
    f_index: Cell<bool>,
    index: RefCell<OptRc<MozillaMar_MarIndex>>,
}
impl KStruct for MozillaMar {
    type Root = MozillaMar;
    type Parent = MozillaMar;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.magic() == vec![0x4du8, 0x41u8, 0x52u8, 0x31u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.ofs_index.borrow_mut() = _io.read_u4be()?;
        *self_rc.file_size.borrow_mut() = _io.read_u8be()?;
        *self_rc.len_signatures.borrow_mut() = _io.read_u4be()?;
        *self_rc.signatures.borrow_mut() = Vec::new();
        let l_signatures = usize::try_from(*self_rc.len_signatures())?;
        for _i in 0_usize..l_signatures {
            let t = Self::read_into::<_, MozillaMar_Signature>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.signatures.borrow_mut().push(t);
        }
        *self_rc.len_additional_sections.borrow_mut() = _io.read_u4be()?;
        *self_rc.additional_sections.borrow_mut() = Vec::new();
        let l_additional_sections = usize::try_from(*self_rc.len_additional_sections())?;
        for _i in 0_usize..l_additional_sections {
            let t = Self::read_into::<_, MozillaMar_AdditionalSection>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.additional_sections.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn index(
        &self
    ) -> KResult<Ref<'_, OptRc<MozillaMar_MarIndex>>> {
        let _io = self._io.borrow();
        if self.f_index.get() {
            return Ok(self.index.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs_index())?)?;
        let t = Self::read_into::<_, MozillaMar_MarIndex>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.index.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.index.borrow())
    }
}
impl MozillaMar {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl MozillaMar {
    pub fn ofs_index(&self) -> Ref<'_, u32> {
        self.ofs_index.borrow()
    }
}
impl MozillaMar {
    pub fn file_size(&self) -> Ref<'_, u64> {
        self.file_size.borrow()
    }
}
impl MozillaMar {
    pub fn len_signatures(&self) -> Ref<'_, u32> {
        self.len_signatures.borrow()
    }
}
impl MozillaMar {
    pub fn signatures(&self) -> Ref<'_, Vec<OptRc<MozillaMar_Signature>>> {
        self.signatures.borrow()
    }
}
impl MozillaMar {
    pub fn len_additional_sections(&self) -> Ref<'_, u32> {
        self.len_additional_sections.borrow()
    }
}
impl MozillaMar {
    pub fn additional_sections(&self) -> Ref<'_, Vec<OptRc<MozillaMar_AdditionalSection>>> {
        self.additional_sections.borrow()
    }
}
impl MozillaMar {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MozillaMar_BlockIdentifiers {
    ProductInformation,
    Unknown(i64),
}

impl TryFrom<i64> for MozillaMar_BlockIdentifiers {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<MozillaMar_BlockIdentifiers> {
        match flag {
            1 => Ok(MozillaMar_BlockIdentifiers::ProductInformation),
            _ => Ok(MozillaMar_BlockIdentifiers::Unknown(flag)),
        }
    }
}

impl From<&MozillaMar_BlockIdentifiers> for i64 {
    fn from(v: &MozillaMar_BlockIdentifiers) -> Self {
        match *v {
            MozillaMar_BlockIdentifiers::ProductInformation => 1,
            MozillaMar_BlockIdentifiers::Unknown(v) => v
        }
    }
}

impl Default for MozillaMar_BlockIdentifiers {
    fn default() -> Self { MozillaMar_BlockIdentifiers::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MozillaMar_SignatureAlgorithms {
    RsaPkcs1Sha1,
    RsaPkcs1Sha384,
    Unknown(i64),
}

impl TryFrom<i64> for MozillaMar_SignatureAlgorithms {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<MozillaMar_SignatureAlgorithms> {
        match flag {
            1 => Ok(MozillaMar_SignatureAlgorithms::RsaPkcs1Sha1),
            2 => Ok(MozillaMar_SignatureAlgorithms::RsaPkcs1Sha384),
            _ => Ok(MozillaMar_SignatureAlgorithms::Unknown(flag)),
        }
    }
}

impl From<&MozillaMar_SignatureAlgorithms> for i64 {
    fn from(v: &MozillaMar_SignatureAlgorithms) -> Self {
        match *v {
            MozillaMar_SignatureAlgorithms::RsaPkcs1Sha1 => 1,
            MozillaMar_SignatureAlgorithms::RsaPkcs1Sha384 => 2,
            MozillaMar_SignatureAlgorithms::Unknown(v) => v
        }
    }
}

impl Default for MozillaMar_SignatureAlgorithms {
    fn default() -> Self { MozillaMar_SignatureAlgorithms::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct MozillaMar_AdditionalSection {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar>,
    pub(crate) _self_shared: SharedType<Self>,
    len_block: RefCell<u32>,
    block_identifier: RefCell<MozillaMar_BlockIdentifiers>,
    bytes: RefCell<Option<MozillaMar_AdditionalSection_Bytes>>,
    _io: RefCell<BytesReader>,
    bytes_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum MozillaMar_AdditionalSection_Bytes {
    MozillaMar_ProductInformationBlock(OptRc<MozillaMar_ProductInformationBlock>),
    Bytes(Vec<u8>),
}
impl TryFrom<&MozillaMar_AdditionalSection_Bytes> for OptRc<MozillaMar_ProductInformationBlock> {
    type Error = KError;
    fn try_from(v: &MozillaMar_AdditionalSection_Bytes) -> Result<Self, Self::Error> {
        if let MozillaMar_AdditionalSection_Bytes::MozillaMar_ProductInformationBlock(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<MozillaMar_ProductInformationBlock>> for MozillaMar_AdditionalSection_Bytes {
    fn from(v: OptRc<MozillaMar_ProductInformationBlock>) -> Self {
        Self::MozillaMar_ProductInformationBlock(v)
    }
}
impl TryFrom<&MozillaMar_AdditionalSection_Bytes> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &MozillaMar_AdditionalSection_Bytes) -> Result<Self, Self::Error> {
        if let MozillaMar_AdditionalSection_Bytes::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for MozillaMar_AdditionalSection_Bytes {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for MozillaMar_AdditionalSection {
    type Root = MozillaMar;
    type Parent = MozillaMar;

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
        *self_rc.len_block.borrow_mut() = _io.read_u4be()?;
        *self_rc.block_identifier.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        match *self_rc.block_identifier() {
            MozillaMar_BlockIdentifiers::ProductInformation => {
                *self_rc.bytes_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.len_block()).saturating_sub(u32::try_from(4_i32)?)).saturating_sub(u32::try_from(4_i32)?))?)?.into();
                let bytes_raw = self_rc.bytes_raw.borrow();
                let _t_bytes_raw_io = BytesReader::from(bytes_raw.clone());
                let t = Self::read_into::<BytesReader, MozillaMar_ProductInformationBlock>(&_t_bytes_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.bytes.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.bytes.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar_AdditionalSection {
}
impl MozillaMar_AdditionalSection {
    pub fn len_block(&self) -> Ref<'_, u32> {
        self.len_block.borrow()
    }
}
impl MozillaMar_AdditionalSection {
    pub fn block_identifier(&self) -> Ref<'_, MozillaMar_BlockIdentifiers> {
        self.block_identifier.borrow()
    }
}
impl MozillaMar_AdditionalSection {
    pub fn bytes(&self) -> Ref<'_, Option<MozillaMar_AdditionalSection_Bytes>> {
        self.bytes.borrow()
    }
}
impl MozillaMar_AdditionalSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl MozillaMar_AdditionalSection {
    pub fn bytes_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bytes_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MozillaMar_IndexEntries {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar_MarIndex>,
    pub(crate) _self_shared: SharedType<Self>,
    index_entry: RefCell<Vec<OptRc<MozillaMar_IndexEntry>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MozillaMar_IndexEntries {
    type Root = MozillaMar;
    type Parent = MozillaMar_MarIndex;

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
        *self_rc.index_entry.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, MozillaMar_IndexEntry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.index_entry.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar_IndexEntries {
}
impl MozillaMar_IndexEntries {
    pub fn index_entry(&self) -> Ref<'_, Vec<OptRc<MozillaMar_IndexEntry>>> {
        self.index_entry.borrow()
    }
}
impl MozillaMar_IndexEntries {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MozillaMar_IndexEntry {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar_IndexEntries>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs_content: RefCell<u32>,
    len_content: RefCell<u32>,
    flags: RefCell<u32>,
    file_name: RefCell<String>,
    _io: RefCell<BytesReader>,
    f_body: Cell<bool>,
    body: RefCell<Vec<u8>>,
}
impl KStruct for MozillaMar_IndexEntry {
    type Root = MozillaMar;
    type Parent = MozillaMar_IndexEntries;

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
        *self_rc.ofs_content.borrow_mut() = _io.read_u4be()?;
        *self_rc.len_content.borrow_mut() = _io.read_u4be()?;
        *self_rc.flags.borrow_mut() = _io.read_u4be()?;
        *self_rc.file_name.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar_IndexEntry {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn body(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        self.f_body.set(true);
        let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
        let _pos = io.pos();
        io.seek(usize::try_from(*self.ofs_content())?)?;
        *self.body.borrow_mut() = io.read_bytes(usize::try_from(*self.len_content())?)?;
        io.seek(_pos)?;
        Ok(self.body.borrow())
    }
}
impl MozillaMar_IndexEntry {
    pub fn ofs_content(&self) -> Ref<'_, u32> {
        self.ofs_content.borrow()
    }
}
impl MozillaMar_IndexEntry {
    pub fn len_content(&self) -> Ref<'_, u32> {
        self.len_content.borrow()
    }
}

/**
 * File permission bits (in standard unix-style format).
 */
impl MozillaMar_IndexEntry {
    pub fn flags(&self) -> Ref<'_, u32> {
        self.flags.borrow()
    }
}
impl MozillaMar_IndexEntry {
    pub fn file_name(&self) -> Ref<'_, String> {
        self.file_name.borrow()
    }
}
impl MozillaMar_IndexEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MozillaMar_MarIndex {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar>,
    pub(crate) _self_shared: SharedType<Self>,
    len_index: RefCell<u32>,
    index_entries: RefCell<OptRc<MozillaMar_IndexEntries>>,
    _io: RefCell<BytesReader>,
    index_entries_raw: RefCell<Vec<u8>>,
}
impl KStruct for MozillaMar_MarIndex {
    type Root = MozillaMar;
    type Parent = MozillaMar;

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
        *self_rc.len_index.borrow_mut() = _io.read_u4be()?;
        let _raw_index_entries = _io.read_bytes(usize::try_from(*self_rc.len_index())?)?;
        *self_rc.index_entries_raw.borrow_mut() = _raw_index_entries.clone();
        let _io_index_entries = BytesReader::from(_raw_index_entries);
        let t = Self::read_into::<BytesReader, MozillaMar_IndexEntries>(&_io_index_entries, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.index_entries.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar_MarIndex {
}
impl MozillaMar_MarIndex {
    pub fn len_index(&self) -> Ref<'_, u32> {
        self.len_index.borrow()
    }
}
impl MozillaMar_MarIndex {
    pub fn index_entries(&self) -> Ref<'_, OptRc<MozillaMar_IndexEntries>> {
        self.index_entries.borrow()
    }
}
impl MozillaMar_MarIndex {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl MozillaMar_MarIndex {
    pub fn index_entries_raw(&self) -> Ref<'_, Vec<u8>> {
        self.index_entries_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MozillaMar_ProductInformationBlock {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar_AdditionalSection>,
    pub(crate) _self_shared: SharedType<Self>,
    mar_channel_name: RefCell<String>,
    product_version: RefCell<String>,
    _io: RefCell<BytesReader>,
    mar_channel_name_raw: RefCell<Vec<u8>>,
    product_version_raw: RefCell<Vec<u8>>,
}
impl KStruct for MozillaMar_ProductInformationBlock {
    type Root = MozillaMar;
    type Parent = MozillaMar_AdditionalSection;

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
        *self_rc.mar_channel_name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(64_usize)?, Some(0), false, None), "UTF-8")?;
        *self_rc.product_version.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(32_usize)?, Some(0), false, None), "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar_ProductInformationBlock {
}
impl MozillaMar_ProductInformationBlock {
    pub fn mar_channel_name(&self) -> Ref<'_, String> {
        self.mar_channel_name.borrow()
    }
}
impl MozillaMar_ProductInformationBlock {
    pub fn product_version(&self) -> Ref<'_, String> {
        self.product_version.borrow()
    }
}
impl MozillaMar_ProductInformationBlock {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl MozillaMar_ProductInformationBlock {
    pub fn mar_channel_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.mar_channel_name_raw.borrow()
    }
}
impl MozillaMar_ProductInformationBlock {
    pub fn product_version_raw(&self) -> Ref<'_, Vec<u8>> {
        self.product_version_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MozillaMar_Signature {
    pub(crate) _root: SharedType<MozillaMar>,
    pub(crate) _parent: SharedType<MozillaMar>,
    pub(crate) _self_shared: SharedType<Self>,
    algorithm: RefCell<MozillaMar_SignatureAlgorithms>,
    len_signature: RefCell<u32>,
    signature: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    signature_raw: RefCell<Vec<u8>>,
}
impl KStruct for MozillaMar_Signature {
    type Root = MozillaMar;
    type Parent = MozillaMar;

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
        *self_rc.algorithm.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        *self_rc.len_signature.borrow_mut() = _io.read_u4be()?;
        *self_rc.signature.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_signature())?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl MozillaMar_Signature {
}
impl MozillaMar_Signature {
    pub fn algorithm(&self) -> Ref<'_, MozillaMar_SignatureAlgorithms> {
        self.algorithm.borrow()
    }
}
impl MozillaMar_Signature {
    pub fn len_signature(&self) -> Ref<'_, u32> {
        self.len_signature.borrow()
    }
}
impl MozillaMar_Signature {
    pub fn signature(&self) -> Ref<'_, Vec<u8>> {
        self.signature.borrow()
    }
}
impl MozillaMar_Signature {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl MozillaMar_Signature {
    pub fn signature_raw(&self) -> Ref<'_, Vec<u8>> {
        self.signature_raw.borrow()
    }
}
