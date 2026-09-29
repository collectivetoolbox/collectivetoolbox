// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Creative Voice File is a container file format for digital audio
 * wave data. Initial revisions were able to support only unsigned
 * 8-bit PCM and ADPCM data, later versions were revised to add support
 * for 16-bit PCM and a-law / u-law formats.
 *
 * This format was actively used in 1990s, around the advent of
 * Creative's sound cards (Sound Blaster family). It was a popular
 * choice for a digital sound container in lots of games and multimedia
 * software due to simplicity and availability of Creative's recording
 * / editing tools.
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    header_size: RefCell<u16>,
    version: RefCell<u16>,
    checksum: RefCell<u16>,
    blocks: RefCell<Vec<OptRc<CreativeVoiceFile_Block>>>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&CreativeVoiceFile> for OptRc<CreativeVoiceFile> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile> for OptRc<CreativeVoiceFile> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile> for CreativeVoiceFile {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile> for &CreativeVoiceFile {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile> for OptRc<CreativeVoiceFile> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile> for &OptRc<CreativeVoiceFile> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(20_usize)?;
        if !(*self_rc.magic() == vec![0x43u8, 0x72u8, 0x65u8, 0x61u8, 0x74u8, 0x69u8, 0x76u8, 0x65u8, 0x20u8, 0x56u8, 0x6fu8, 0x69u8, 0x63u8, 0x65u8, 0x20u8, 0x46u8, 0x69u8, 0x6cu8, 0x65u8, 0x1au8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.header_size.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.version.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.checksum.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.blocks.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, CreativeVoiceFile_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.blocks.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile {
}
impl CreativeVoiceFile {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}

/**
 * Total size of this main header (usually 0x001A)
 */
impl CreativeVoiceFile {
    pub fn header_size(&self) -> Ref<'_, u16> {
        self.header_size.borrow()
    }
}
impl CreativeVoiceFile {
    pub fn version(&self) -> Ref<'_, u16> {
        self.version.borrow()
    }
}

/**
 * Checksum: this must be equal to ~version + 0x1234
 */
impl CreativeVoiceFile {
    pub fn checksum(&self) -> Ref<'_, u16> {
        self.checksum.borrow()
    }
}

/**
 * Series of blocks that contain the actual audio data
 */
impl CreativeVoiceFile {
    pub fn blocks(&self) -> Ref<'_, Vec<OptRc<CreativeVoiceFile_Block>>> {
        self.blocks.borrow()
    }
}
impl CreativeVoiceFile {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CreativeVoiceFile_BlockTypes {
    Terminator,
    SoundData,
    SoundDataCont,
    Silence,
    Marker,
    Text,
    RepeatStart,
    RepeatEnd,
    ExtraInfo,
    SoundDataNew,
    Unknown(i64),
}

impl TryFrom<i64> for CreativeVoiceFile_BlockTypes {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<CreativeVoiceFile_BlockTypes> {
        match flag {
            0 => Ok(CreativeVoiceFile_BlockTypes::Terminator),
            1 => Ok(CreativeVoiceFile_BlockTypes::SoundData),
            2 => Ok(CreativeVoiceFile_BlockTypes::SoundDataCont),
            3 => Ok(CreativeVoiceFile_BlockTypes::Silence),
            4 => Ok(CreativeVoiceFile_BlockTypes::Marker),
            5 => Ok(CreativeVoiceFile_BlockTypes::Text),
            6 => Ok(CreativeVoiceFile_BlockTypes::RepeatStart),
            7 => Ok(CreativeVoiceFile_BlockTypes::RepeatEnd),
            8 => Ok(CreativeVoiceFile_BlockTypes::ExtraInfo),
            9 => Ok(CreativeVoiceFile_BlockTypes::SoundDataNew),
            _ => Ok(CreativeVoiceFile_BlockTypes::Unknown(flag)),
        }
    }
}

impl From<&CreativeVoiceFile_BlockTypes> for i64 {
    fn from(v: &CreativeVoiceFile_BlockTypes) -> Self {
        match *v {
            CreativeVoiceFile_BlockTypes::Terminator => 0,
            CreativeVoiceFile_BlockTypes::SoundData => 1,
            CreativeVoiceFile_BlockTypes::SoundDataCont => 2,
            CreativeVoiceFile_BlockTypes::Silence => 3,
            CreativeVoiceFile_BlockTypes::Marker => 4,
            CreativeVoiceFile_BlockTypes::Text => 5,
            CreativeVoiceFile_BlockTypes::RepeatStart => 6,
            CreativeVoiceFile_BlockTypes::RepeatEnd => 7,
            CreativeVoiceFile_BlockTypes::ExtraInfo => 8,
            CreativeVoiceFile_BlockTypes::SoundDataNew => 9,
            CreativeVoiceFile_BlockTypes::Unknown(v) => v
        }
    }
}

impl Default for CreativeVoiceFile_BlockTypes {
    fn default() -> Self { CreativeVoiceFile_BlockTypes::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CreativeVoiceFile_Codecs {
    Pcm8bitUnsigned,
    Adpcm4bit,
    Adpcm26bit,
    Adpcm2Bit,
    Pcm16bitSigned,
    Alaw,
    Ulaw,
    Adpcm4To16bit,
    Unknown(i64),
}

impl TryFrom<i64> for CreativeVoiceFile_Codecs {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<CreativeVoiceFile_Codecs> {
        match flag {
            0 => Ok(CreativeVoiceFile_Codecs::Pcm8bitUnsigned),
            1 => Ok(CreativeVoiceFile_Codecs::Adpcm4bit),
            2 => Ok(CreativeVoiceFile_Codecs::Adpcm26bit),
            3 => Ok(CreativeVoiceFile_Codecs::Adpcm2Bit),
            4 => Ok(CreativeVoiceFile_Codecs::Pcm16bitSigned),
            6 => Ok(CreativeVoiceFile_Codecs::Alaw),
            7 => Ok(CreativeVoiceFile_Codecs::Ulaw),
            512 => Ok(CreativeVoiceFile_Codecs::Adpcm4To16bit),
            _ => Ok(CreativeVoiceFile_Codecs::Unknown(flag)),
        }
    }
}

impl From<&CreativeVoiceFile_Codecs> for i64 {
    fn from(v: &CreativeVoiceFile_Codecs) -> Self {
        match *v {
            CreativeVoiceFile_Codecs::Pcm8bitUnsigned => 0,
            CreativeVoiceFile_Codecs::Adpcm4bit => 1,
            CreativeVoiceFile_Codecs::Adpcm26bit => 2,
            CreativeVoiceFile_Codecs::Adpcm2Bit => 3,
            CreativeVoiceFile_Codecs::Pcm16bitSigned => 4,
            CreativeVoiceFile_Codecs::Alaw => 6,
            CreativeVoiceFile_Codecs::Ulaw => 7,
            CreativeVoiceFile_Codecs::Adpcm4To16bit => 512,
            CreativeVoiceFile_Codecs::Unknown(v) => v
        }
    }
}

impl Default for CreativeVoiceFile_Codecs {
    fn default() -> Self { CreativeVoiceFile_Codecs::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_Block {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile>,
    pub(crate) _self_shared: SharedType<Self>,
    block_type: RefCell<CreativeVoiceFile_BlockTypes>,
    body_size1: RefCell<u16>,
    body_size2: RefCell<u8>,
    body: RefCell<Option<CreativeVoiceFile_Block_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
    f_body_size: Cell<bool>,
    body_size: RefCell<i32>,
}
#[derive(Debug, Clone)]
pub enum CreativeVoiceFile_Block_Body {
    CreativeVoiceFile_BlockExtraInfo(OptRc<CreativeVoiceFile_BlockExtraInfo>),
    CreativeVoiceFile_BlockMarker(OptRc<CreativeVoiceFile_BlockMarker>),
    CreativeVoiceFile_BlockRepeatStart(OptRc<CreativeVoiceFile_BlockRepeatStart>),
    CreativeVoiceFile_BlockSilence(OptRc<CreativeVoiceFile_BlockSilence>),
    CreativeVoiceFile_BlockSoundData(OptRc<CreativeVoiceFile_BlockSoundData>),
    CreativeVoiceFile_BlockSoundDataNew(OptRc<CreativeVoiceFile_BlockSoundDataNew>),
    Bytes(Vec<u8>),
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockExtraInfo> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::CreativeVoiceFile_BlockExtraInfo(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockExtraInfo> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockExtraInfo> for CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockExtraInfo>, KError> {
        OptRc::<CreativeVoiceFile_BlockExtraInfo>::try_from(self)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockExtraInfo> for &CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockExtraInfo>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<CreativeVoiceFile_BlockExtraInfo>> for CreativeVoiceFile_Block_Body {
    fn from(v: OptRc<CreativeVoiceFile_BlockExtraInfo>) -> Self {
        Self::CreativeVoiceFile_BlockExtraInfo(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockMarker> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::CreativeVoiceFile_BlockMarker(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockMarker> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockMarker> for CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockMarker>, KError> {
        OptRc::<CreativeVoiceFile_BlockMarker>::try_from(self)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockMarker> for &CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockMarker>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<CreativeVoiceFile_BlockMarker>> for CreativeVoiceFile_Block_Body {
    fn from(v: OptRc<CreativeVoiceFile_BlockMarker>) -> Self {
        Self::CreativeVoiceFile_BlockMarker(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockRepeatStart> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::CreativeVoiceFile_BlockRepeatStart(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockRepeatStart> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockRepeatStart> for CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockRepeatStart>, KError> {
        OptRc::<CreativeVoiceFile_BlockRepeatStart>::try_from(self)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockRepeatStart> for &CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockRepeatStart>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<CreativeVoiceFile_BlockRepeatStart>> for CreativeVoiceFile_Block_Body {
    fn from(v: OptRc<CreativeVoiceFile_BlockRepeatStart>) -> Self {
        Self::CreativeVoiceFile_BlockRepeatStart(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockSilence> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::CreativeVoiceFile_BlockSilence(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockSilence> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSilence> for CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSilence>, KError> {
        OptRc::<CreativeVoiceFile_BlockSilence>::try_from(self)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSilence> for &CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSilence>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<CreativeVoiceFile_BlockSilence>> for CreativeVoiceFile_Block_Body {
    fn from(v: OptRc<CreativeVoiceFile_BlockSilence>) -> Self {
        Self::CreativeVoiceFile_BlockSilence(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockSoundData> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::CreativeVoiceFile_BlockSoundData(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockSoundData> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundData> for CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundData>, KError> {
        OptRc::<CreativeVoiceFile_BlockSoundData>::try_from(self)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundData> for &CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundData>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<CreativeVoiceFile_BlockSoundData>> for CreativeVoiceFile_Block_Body {
    fn from(v: OptRc<CreativeVoiceFile_BlockSoundData>) -> Self {
        Self::CreativeVoiceFile_BlockSoundData(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockSoundDataNew> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::CreativeVoiceFile_BlockSoundDataNew(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for OptRc<CreativeVoiceFile_BlockSoundDataNew> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundDataNew> for CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundDataNew>, KError> {
        OptRc::<CreativeVoiceFile_BlockSoundDataNew>::try_from(self)
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundDataNew> for &CreativeVoiceFile_Block_Body {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundDataNew>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<CreativeVoiceFile_BlockSoundDataNew>> for CreativeVoiceFile_Block_Body {
    fn from(v: OptRc<CreativeVoiceFile_BlockSoundDataNew>) -> Self {
        Self::CreativeVoiceFile_BlockSoundDataNew(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        if let CreativeVoiceFile_Block_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&CreativeVoiceFile_Block_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl From<Vec<u8>> for CreativeVoiceFile_Block_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl TryFrom<&CreativeVoiceFile_Block> for OptRc<CreativeVoiceFile_Block> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_Block) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_Block> for OptRc<CreativeVoiceFile_Block> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_Block) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_Block> for CreativeVoiceFile_Block {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_Block>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_Block> for &CreativeVoiceFile_Block {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_Block>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_Block> for OptRc<CreativeVoiceFile_Block> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_Block>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_Block> for &OptRc<CreativeVoiceFile_Block> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_Block>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_Block {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile;

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
        *self_rc.block_type.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = _io.clone();
        if *self_rc.block_type() != CreativeVoiceFile_BlockTypes::Terminator {
            *self_rc.body_size1.borrow_mut() = _io.read_u2le()?;
            *self_rc._io.borrow_mut() = _io.clone();
        }
        *self_rc._io.borrow_mut() = _io.clone();
        if *self_rc.block_type() != CreativeVoiceFile_BlockTypes::Terminator {
            *self_rc.body_size2.borrow_mut() = _io.read_u1()?;
            *self_rc._io.borrow_mut() = _io.clone();
        }
        *self_rc._io.borrow_mut() = _io.clone();
        if *self_rc.block_type() != CreativeVoiceFile_BlockTypes::Terminator {
            match *self_rc.block_type() {
                CreativeVoiceFile_BlockTypes::ExtraInfo => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, CreativeVoiceFile_BlockExtraInfo>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                CreativeVoiceFile_BlockTypes::Marker => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, CreativeVoiceFile_BlockMarker>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                CreativeVoiceFile_BlockTypes::RepeatStart => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, CreativeVoiceFile_BlockRepeatStart>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                CreativeVoiceFile_BlockTypes::Silence => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, CreativeVoiceFile_BlockSilence>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                CreativeVoiceFile_BlockTypes::SoundData => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, CreativeVoiceFile_BlockSoundData>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                CreativeVoiceFile_BlockTypes::SoundDataNew => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, CreativeVoiceFile_BlockSoundDataNew>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                _ => {
                    *self_rc.body.borrow_mut() = Some(_io.read_bytes(usize::try_from(*self_rc.body_size()?)?)?.into());
                }
            }
            *self_rc._io.borrow_mut() = _io.clone();
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_Block {

    /**
     * body_size is a 24-bit little-endian integer, so we're
     * emulating that by adding two standard-sized integers
     * (body_size1 and body_size2).
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn body_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_body_size.get() {
            return Ok(self.body_size.borrow());
        }
        self.f_body_size.set(true);
        if *self.block_type() != CreativeVoiceFile_BlockTypes::Terminator {
            *self.body_size.borrow_mut() = ((i32::from(*self.body_size1())).saturating_add((i32::from(*self.body_size2())).wrapping_shl(16_u32))).try_into()?;
        }
        Ok(self.body_size.borrow())
    }
}

/**
 * Byte that determines type of block content
 */
impl CreativeVoiceFile_Block {
    pub fn block_type(&self) -> Ref<'_, CreativeVoiceFile_BlockTypes> {
        self.block_type.borrow()
    }
}
impl CreativeVoiceFile_Block {
    pub fn body_size1(&self) -> Ref<'_, u16> {
        self.body_size1.borrow()
    }
}
impl CreativeVoiceFile_Block {
    pub fn body_size2(&self) -> Ref<'_, u8> {
        self.body_size2.borrow()
    }
}

/**
 * Block body, type depends on block type byte
 */
impl CreativeVoiceFile_Block {
    pub fn body(&self) -> Ref<'_, Option<CreativeVoiceFile_Block_Body>> {
        self.body.borrow()
    }
}
impl CreativeVoiceFile_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}
impl CreativeVoiceFile_Block {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

/**
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice#Block_type_0x08:_Extra_info> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_BlockExtraInfo {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    freq_div: RefCell<u16>,
    codec: RefCell<CreativeVoiceFile_Codecs>,
    num_channels_1: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_num_channels: Cell<bool>,
    num_channels: RefCell<i32>,
    f_sample_rate: Cell<bool>,
    sample_rate: RefCell<f64>,
}
impl TryFrom<&CreativeVoiceFile_BlockExtraInfo> for OptRc<CreativeVoiceFile_BlockExtraInfo> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_BlockExtraInfo) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_BlockExtraInfo> for OptRc<CreativeVoiceFile_BlockExtraInfo> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_BlockExtraInfo) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockExtraInfo> for CreativeVoiceFile_BlockExtraInfo {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockExtraInfo>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockExtraInfo> for &CreativeVoiceFile_BlockExtraInfo {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockExtraInfo>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockExtraInfo> for OptRc<CreativeVoiceFile_BlockExtraInfo> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockExtraInfo>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockExtraInfo> for &OptRc<CreativeVoiceFile_BlockExtraInfo> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockExtraInfo>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_BlockExtraInfo {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile_Block;

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
        *self_rc.freq_div.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.codec.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.num_channels_1.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_BlockExtraInfo {

    /**
     * Number of channels (1 = mono, 2 = stereo)
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn num_channels(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_num_channels.get() {
            return Ok(self.num_channels.borrow());
        }
        self.f_num_channels.set(true);
        *self.num_channels.borrow_mut() = ((i32::from(*self.num_channels_1())).saturating_add(1_i32)).try_into()?;
        Ok(self.num_channels.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sample_rate(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_sample_rate.get() {
            return Ok(self.sample_rate.borrow());
        }
        self.f_sample_rate.set(true);
        *self.sample_rate.borrow_mut() = (((256000000.0) / (to_f64((*self.num_channels()?).saturating_mul((65536_i32).saturating_sub(i32::from(*self.freq_div()))))))).try_into()?;
        Ok(self.sample_rate.borrow())
    }
}

/**
 * Frequency divisor
 */
impl CreativeVoiceFile_BlockExtraInfo {
    pub fn freq_div(&self) -> Ref<'_, u16> {
        self.freq_div.borrow()
    }
}
impl CreativeVoiceFile_BlockExtraInfo {
    pub fn codec(&self) -> Ref<'_, CreativeVoiceFile_Codecs> {
        self.codec.borrow()
    }
}

/**
 * Number of channels minus 1 (0 = mono, 1 = stereo)
 */
impl CreativeVoiceFile_BlockExtraInfo {
    pub fn num_channels_1(&self) -> Ref<'_, u8> {
        self.num_channels_1.borrow()
    }
}
impl CreativeVoiceFile_BlockExtraInfo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}

/**
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice#Block_type_0x04:_Marker> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_BlockMarker {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    marker_id: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&CreativeVoiceFile_BlockMarker> for OptRc<CreativeVoiceFile_BlockMarker> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_BlockMarker) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_BlockMarker> for OptRc<CreativeVoiceFile_BlockMarker> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_BlockMarker) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockMarker> for CreativeVoiceFile_BlockMarker {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockMarker>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockMarker> for &CreativeVoiceFile_BlockMarker {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockMarker>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockMarker> for OptRc<CreativeVoiceFile_BlockMarker> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockMarker>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockMarker> for &OptRc<CreativeVoiceFile_BlockMarker> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockMarker>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_BlockMarker {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile_Block;

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
        *self_rc.marker_id.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_BlockMarker {
}

/**
 * Marker ID
 */
impl CreativeVoiceFile_BlockMarker {
    pub fn marker_id(&self) -> Ref<'_, u16> {
        self.marker_id.borrow()
    }
}
impl CreativeVoiceFile_BlockMarker {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}

/**
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice#Block_type_0x06:_Repeat_start> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_BlockRepeatStart {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    repeat_count_1: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&CreativeVoiceFile_BlockRepeatStart> for OptRc<CreativeVoiceFile_BlockRepeatStart> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_BlockRepeatStart) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_BlockRepeatStart> for OptRc<CreativeVoiceFile_BlockRepeatStart> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_BlockRepeatStart) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockRepeatStart> for CreativeVoiceFile_BlockRepeatStart {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockRepeatStart>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockRepeatStart> for &CreativeVoiceFile_BlockRepeatStart {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockRepeatStart>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockRepeatStart> for OptRc<CreativeVoiceFile_BlockRepeatStart> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockRepeatStart>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockRepeatStart> for &OptRc<CreativeVoiceFile_BlockRepeatStart> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockRepeatStart>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_BlockRepeatStart {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile_Block;

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
        *self_rc.repeat_count_1.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_BlockRepeatStart {
}

/**
 * Number of repetitions minus 1; 0xffff means infinite repetitions
 */
impl CreativeVoiceFile_BlockRepeatStart {
    pub fn repeat_count_1(&self) -> Ref<'_, u16> {
        self.repeat_count_1.borrow()
    }
}
impl CreativeVoiceFile_BlockRepeatStart {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}

/**
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice#Block_type_0x03:_Silence> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_BlockSilence {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    duration_samples: RefCell<u16>,
    freq_div: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_duration_sec: Cell<bool>,
    duration_sec: RefCell<f64>,
    f_sample_rate: Cell<bool>,
    sample_rate: RefCell<f64>,
}
impl TryFrom<&CreativeVoiceFile_BlockSilence> for OptRc<CreativeVoiceFile_BlockSilence> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_BlockSilence) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_BlockSilence> for OptRc<CreativeVoiceFile_BlockSilence> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_BlockSilence) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSilence> for CreativeVoiceFile_BlockSilence {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSilence>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSilence> for &CreativeVoiceFile_BlockSilence {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSilence>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSilence> for OptRc<CreativeVoiceFile_BlockSilence> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSilence>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSilence> for &OptRc<CreativeVoiceFile_BlockSilence> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSilence>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_BlockSilence {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile_Block;

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
        *self_rc.duration_samples.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.freq_div.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_BlockSilence {

    /**
     * Duration of silence, in seconds
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn duration_sec(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_duration_sec.get() {
            return Ok(self.duration_sec.borrow());
        }
        self.f_duration_sec.set(true);
        *self.duration_sec.borrow_mut() = (((to_f64(*self.duration_samples())) / (*self.sample_rate()?))).try_into()?;
        Ok(self.duration_sec.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sample_rate(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_sample_rate.get() {
            return Ok(self.sample_rate.borrow());
        }
        self.f_sample_rate.set(true);
        *self.sample_rate.borrow_mut() = (((1000000.0) / (to_f64((256_i32).saturating_sub(i32::from(*self.freq_div())))))).try_into()?;
        Ok(self.sample_rate.borrow())
    }
}

/**
 * Duration of silence, in samples
 */
impl CreativeVoiceFile_BlockSilence {
    pub fn duration_samples(&self) -> Ref<'_, u16> {
        self.duration_samples.borrow()
    }
}

/**
 * Frequency divisor, used to determine sample rate
 */
impl CreativeVoiceFile_BlockSilence {
    pub fn freq_div(&self) -> Ref<'_, u8> {
        self.freq_div.borrow()
    }
}
impl CreativeVoiceFile_BlockSilence {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}

/**
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice#Block_type_0x01:_Sound_data> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_BlockSoundData {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    freq_div: RefCell<u8>,
    codec: RefCell<CreativeVoiceFile_Codecs>,
    wave: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_sample_rate: Cell<bool>,
    sample_rate: RefCell<f64>,
}
impl TryFrom<&CreativeVoiceFile_BlockSoundData> for OptRc<CreativeVoiceFile_BlockSoundData> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_BlockSoundData) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_BlockSoundData> for OptRc<CreativeVoiceFile_BlockSoundData> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_BlockSoundData) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundData> for CreativeVoiceFile_BlockSoundData {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundData>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundData> for &CreativeVoiceFile_BlockSoundData {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundData>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundData> for OptRc<CreativeVoiceFile_BlockSoundData> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundData>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundData> for &OptRc<CreativeVoiceFile_BlockSoundData> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundData>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_BlockSoundData {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile_Block;

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
        *self_rc.freq_div.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.codec.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.wave.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_BlockSoundData {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sample_rate(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_sample_rate.get() {
            return Ok(self.sample_rate.borrow());
        }
        self.f_sample_rate.set(true);
        *self.sample_rate.borrow_mut() = (((1000000.0) / (to_f64((256_i32).saturating_sub(i32::from(*self.freq_div())))))).try_into()?;
        Ok(self.sample_rate.borrow())
    }
}

/**
 * Frequency divisor, used to determine sample rate
 */
impl CreativeVoiceFile_BlockSoundData {
    pub fn freq_div(&self) -> Ref<'_, u8> {
        self.freq_div.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundData {
    pub fn codec(&self) -> Ref<'_, CreativeVoiceFile_Codecs> {
        self.codec.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundData {
    pub fn wave(&self) -> Ref<'_, Vec<u8>> {
        self.wave.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundData {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}

/**
 * \sa <https://wiki.multimedia.cx/index.php?title=Creative_Voice#Block_type_0x09:_Sound_data_.28New_format.29> Source
 */

#[derive(Default, Debug, Clone)]
pub struct CreativeVoiceFile_BlockSoundDataNew {
    pub(crate) _root: SharedType<CreativeVoiceFile>,
    pub(crate) _parent: SharedType<CreativeVoiceFile_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    sample_rate: RefCell<u32>,
    bits_per_sample: RefCell<u8>,
    num_channels: RefCell<u8>,
    codec: RefCell<CreativeVoiceFile_Codecs>,
    reserved: RefCell<Vec<u8>>,
    wave: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    reserved_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&CreativeVoiceFile_BlockSoundDataNew> for OptRc<CreativeVoiceFile_BlockSoundDataNew> {
    type Error = KError;
    fn try_from(v: &CreativeVoiceFile_BlockSoundDataNew) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&CreativeVoiceFile_BlockSoundDataNew> for OptRc<CreativeVoiceFile_BlockSoundDataNew> {
    type Error = KError;
    fn try_from(v: &&CreativeVoiceFile_BlockSoundDataNew) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundDataNew> for CreativeVoiceFile_BlockSoundDataNew {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundDataNew>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundDataNew> for &CreativeVoiceFile_BlockSoundDataNew {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundDataNew>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundDataNew> for OptRc<CreativeVoiceFile_BlockSoundDataNew> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundDataNew>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<CreativeVoiceFile_BlockSoundDataNew> for &OptRc<CreativeVoiceFile_BlockSoundDataNew> {
    fn downcast_optrc(&self) -> Result<OptRc<CreativeVoiceFile_BlockSoundDataNew>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for CreativeVoiceFile_BlockSoundDataNew {
    type Root = CreativeVoiceFile;
    type Parent = CreativeVoiceFile_Block;

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
        *self_rc.sample_rate.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.bits_per_sample.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.num_channels.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.codec.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.reserved.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.wave.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn sample_rate(&self) -> Ref<'_, u32> {
        self.sample_rate.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn bits_per_sample(&self) -> Ref<'_, u8> {
        self.bits_per_sample.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn num_channels(&self) -> Ref<'_, u8> {
        self.num_channels.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn codec(&self) -> Ref<'_, CreativeVoiceFile_Codecs> {
        self.codec.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn wave(&self) -> Ref<'_, Vec<u8>> {
        self.wave.borrow()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}
impl CreativeVoiceFile_BlockSoundDataNew {
    pub fn reserved_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved_raw.borrow()
    }
}
