// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Bitmap font format for the GRUB 2 bootloader.
 * \sa <https://grub.gibibit.com/New_font_format> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Grub2Font {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    sections: RefCell<Vec<OptRc<Grub2Font_Section>>>,
    _io: RefCell<BytesReader>,
    magic_raw: RefCell<Vec<u8>>,
}
impl KStruct for Grub2Font {
    type Root = Grub2Font;
    type Parent = Grub2Font;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(12_usize)?;
        if !(*self_rc.magic() == vec![0x46u8, 0x49u8, 0x4cu8, 0x45u8, 0x0u8, 0x0u8, 0x0u8, 0x4u8, 0x50u8, 0x46u8, 0x46u8, 0x32u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.sections.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, Grub2Font_Section>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.sections.borrow_mut().push(t);
                let _t_sections = self_rc.sections.borrow();
                let Some(_tmpa) = _t_sections.last() else { break; };
                _i = _i.saturating_add(1);
                if (_tmpa.section_type().as_str() == "DATA") { break; }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font {
}
impl Grub2Font {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}

/**
 * The "DATA" section acts as a terminator. The documentation says:
 * "A marker that indicates the remainder of the file is data accessed
 * via the character index (CHIX) section. When reading this font file,
 * the rest of the file can be ignored when scanning the sections."
 */
impl Grub2Font {
    pub fn sections(&self) -> Ref<'_, Vec<OptRc<Grub2Font_Section>>> {
        self.sections.borrow()
    }
}
impl Grub2Font {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Grub2Font {
    pub fn magic_raw(&self) -> Ref<'_, Vec<u8>> {
        self.magic_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_AsceSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    ascent_in_pixels: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_AsceSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.ascent_in_pixels.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_AsceSection {
}
impl Grub2Font_AsceSection {
    pub fn ascent_in_pixels(&self) -> Ref<'_, u16> {
        self.ascent_in_pixels.borrow()
    }
}
impl Grub2Font_AsceSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_ChixSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    characters: RefCell<Vec<OptRc<Grub2Font_ChixSection_Character>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_ChixSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.characters.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Grub2Font_ChixSection_Character>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.characters.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_ChixSection {
}
impl Grub2Font_ChixSection {
    pub fn characters(&self) -> Ref<'_, Vec<OptRc<Grub2Font_ChixSection_Character>>> {
        self.characters.borrow()
    }
}
impl Grub2Font_ChixSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_ChixSection_Character {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_ChixSection>,
    pub(crate) _self_shared: SharedType<Self>,
    code_point: RefCell<u32>,
    flags: RefCell<u8>,
    ofs_definition: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_definition: Cell<bool>,
    definition: RefCell<OptRc<Grub2Font_ChixSection_CharacterDefinition>>,
}
impl KStruct for Grub2Font_ChixSection_Character {
    type Root = Grub2Font;
    type Parent = Grub2Font_ChixSection;

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
        *self_rc.code_point.borrow_mut() = _io.read_u4be()?;
        *self_rc.flags.borrow_mut() = _io.read_u1()?;
        *self_rc.ofs_definition.borrow_mut() = _io.read_u4be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_ChixSection_Character {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn definition(
        &self
    ) -> KResult<Ref<'_, OptRc<Grub2Font_ChixSection_CharacterDefinition>>> {
        let _io = self._io.borrow();
        if self.f_definition.get() {
            return Ok(self.definition.borrow());
        }
        let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
        let _pos = io.pos();
        io.seek(usize::try_from(*self.ofs_definition())?)?;
        let t = Self::read_into::<_, Grub2Font_ChixSection_CharacterDefinition>(&io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.definition.borrow_mut() = t;
        io.seek(_pos)?;
        Ok(self.definition.borrow())
    }
}

/**
 * Unicode code point
 */
impl Grub2Font_ChixSection_Character {
    pub fn code_point(&self) -> Ref<'_, u32> {
        self.code_point.borrow()
    }
}
impl Grub2Font_ChixSection_Character {
    pub fn flags(&self) -> Ref<'_, u8> {
        self.flags.borrow()
    }
}
impl Grub2Font_ChixSection_Character {
    pub fn ofs_definition(&self) -> Ref<'_, u32> {
        self.ofs_definition.borrow()
    }
}
impl Grub2Font_ChixSection_Character {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_ChixSection_CharacterDefinition {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_ChixSection_Character>,
    pub(crate) _self_shared: SharedType<Self>,
    width: RefCell<u16>,
    height: RefCell<u16>,
    x_offset: RefCell<i16>,
    y_offset: RefCell<i16>,
    device_width: RefCell<i16>,
    bitmap_data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    bitmap_data_raw: RefCell<Vec<u8>>,
}
impl KStruct for Grub2Font_ChixSection_CharacterDefinition {
    type Root = Grub2Font;
    type Parent = Grub2Font_ChixSection_Character;

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
        *self_rc.width.borrow_mut() = _io.read_u2be()?;
        *self_rc.height.borrow_mut() = _io.read_u2be()?;
        *self_rc.x_offset.borrow_mut() = _io.read_s2be()?;
        *self_rc.y_offset.borrow_mut() = _io.read_s2be()?;
        *self_rc.device_width.borrow_mut() = _io.read_s2be()?;
        *self_rc.bitmap_data.borrow_mut() = _io.read_bytes(usize::try_from(div_floor(i64::from((i32::from((*self_rc.width()).saturating_mul(*self_rc.height()))).saturating_add(7_i32)), 8_i64)?)?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn width(&self) -> Ref<'_, u16> {
        self.width.borrow()
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn height(&self) -> Ref<'_, u16> {
        self.height.borrow()
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn x_offset(&self) -> Ref<'_, i16> {
        self.x_offset.borrow()
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn y_offset(&self) -> Ref<'_, i16> {
        self.y_offset.borrow()
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn device_width(&self) -> Ref<'_, i16> {
        self.device_width.borrow()
    }
}

/**
 * A two-dimensional bitmap, one bit per pixel. It is organized as
 * row-major, top-down, left-to-right. The most significant bit of
 * each byte corresponds to the leftmost or uppermost pixel from all
 * bits of the byte. If a bit is set (1, `true`), the pixel is set to
 * the font color, if a bit is clear (0, `false`), the pixel is
 * transparent.
 *
 * Rows are **not** padded to byte boundaries (i.e., a
 * single byte may contain bits belonging to multiple rows). The last
 * byte of the bitmap _is_ padded with zero bits at all unused least
 * significant bit positions so that the bitmap ends on a byte
 * boundary.
 */
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn bitmap_data(&self) -> Ref<'_, Vec<u8>> {
        self.bitmap_data.borrow()
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Grub2Font_ChixSection_CharacterDefinition {
    pub fn bitmap_data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bitmap_data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_DescSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    descent_in_pixels: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_DescSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.descent_in_pixels.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_DescSection {
}
impl Grub2Font_DescSection {
    pub fn descent_in_pixels(&self) -> Ref<'_, u16> {
        self.descent_in_pixels.borrow()
    }
}
impl Grub2Font_DescSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_FamiSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    font_family_name: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_FamiSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.font_family_name.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_FamiSection {
}
impl Grub2Font_FamiSection {
    pub fn font_family_name(&self) -> Ref<'_, String> {
        self.font_family_name.borrow()
    }
}
impl Grub2Font_FamiSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_MaxhSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    maximum_character_height: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_MaxhSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.maximum_character_height.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_MaxhSection {
}
impl Grub2Font_MaxhSection {
    pub fn maximum_character_height(&self) -> Ref<'_, u16> {
        self.maximum_character_height.borrow()
    }
}
impl Grub2Font_MaxhSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_MaxwSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    maximum_character_width: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_MaxwSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.maximum_character_width.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_MaxwSection {
}
impl Grub2Font_MaxwSection {
    pub fn maximum_character_width(&self) -> Ref<'_, u16> {
        self.maximum_character_width.borrow()
    }
}
impl Grub2Font_MaxwSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_NameSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    font_name: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_NameSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.font_name.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_NameSection {
}
impl Grub2Font_NameSection {
    pub fn font_name(&self) -> Ref<'_, String> {
        self.font_name.borrow()
    }
}
impl Grub2Font_NameSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_PtszSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    font_point_size: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_PtszSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.font_point_size.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_PtszSection {
}
impl Grub2Font_PtszSection {
    pub fn font_point_size(&self) -> Ref<'_, u16> {
        self.font_point_size.borrow()
    }
}
impl Grub2Font_PtszSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_Section {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font>,
    pub(crate) _self_shared: SharedType<Self>,
    section_type: RefCell<String>,
    len_body: RefCell<u32>,
    body: RefCell<Option<Grub2Font_Section_Body>>,
    _io: RefCell<BytesReader>,
    section_type_raw: RefCell<Vec<u8>>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Grub2Font_Section_Body {
    Grub2Font_AsceSection(OptRc<Grub2Font_AsceSection>),
    Grub2Font_ChixSection(OptRc<Grub2Font_ChixSection>),
    Grub2Font_DescSection(OptRc<Grub2Font_DescSection>),
    Grub2Font_FamiSection(OptRc<Grub2Font_FamiSection>),
    Grub2Font_MaxhSection(OptRc<Grub2Font_MaxhSection>),
    Grub2Font_MaxwSection(OptRc<Grub2Font_MaxwSection>),
    Grub2Font_NameSection(OptRc<Grub2Font_NameSection>),
    Grub2Font_PtszSection(OptRc<Grub2Font_PtszSection>),
    Grub2Font_SlanSection(OptRc<Grub2Font_SlanSection>),
    Grub2Font_WeigSection(OptRc<Grub2Font_WeigSection>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_AsceSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_AsceSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_AsceSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_AsceSection>) -> Self {
        Self::Grub2Font_AsceSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_ChixSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_ChixSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_ChixSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_ChixSection>) -> Self {
        Self::Grub2Font_ChixSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_DescSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_DescSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_DescSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_DescSection>) -> Self {
        Self::Grub2Font_DescSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_FamiSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_FamiSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_FamiSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_FamiSection>) -> Self {
        Self::Grub2Font_FamiSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_MaxhSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_MaxhSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_MaxhSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_MaxhSection>) -> Self {
        Self::Grub2Font_MaxhSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_MaxwSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_MaxwSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_MaxwSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_MaxwSection>) -> Self {
        Self::Grub2Font_MaxwSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_NameSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_NameSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_NameSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_NameSection>) -> Self {
        Self::Grub2Font_NameSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_PtszSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_PtszSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_PtszSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_PtszSection>) -> Self {
        Self::Grub2Font_PtszSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_SlanSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_SlanSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_SlanSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_SlanSection>) -> Self {
        Self::Grub2Font_SlanSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for OptRc<Grub2Font_WeigSection> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Grub2Font_WeigSection(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Grub2Font_WeigSection>> for Grub2Font_Section_Body {
    fn from(v: OptRc<Grub2Font_WeigSection>) -> Self {
        Self::Grub2Font_WeigSection(v)
    }
}
impl TryFrom<&Grub2Font_Section_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Grub2Font_Section_Body) -> Result<Self, Self::Error> {
        if let Grub2Font_Section_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Grub2Font_Section_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Grub2Font_Section {
    type Root = Grub2Font;
    type Parent = Grub2Font;

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
        *self_rc.section_type.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        *self_rc.len_body.borrow_mut() = _io.read_u4be()?;
        if (self_rc.section_type().as_str() != "DATA") {
            match self_rc.section_type().as_str() {
                "ASCE" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_AsceSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "CHIX" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_ChixSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "DESC" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_DescSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "FAMI" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_FamiSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "MAXH" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_MaxhSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "MAXW" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_MaxwSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "NAME" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_NameSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "PTSZ" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_PtszSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "SLAN" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_SlanSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                "WEIG" => {
                    *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?.into();
                    let body_raw = self_rc.body_raw.borrow();
                    let _t_body_raw_io = BytesReader::from(body_raw.clone());
                    let t = Self::read_into::<BytesReader, Grub2Font_WeigSection>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.body.borrow_mut() = Some(t);
                }
                _ => {
                    *self_rc.body.borrow_mut() = Some(_io.read_bytes_full()?.into());
                }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_Section {
}
impl Grub2Font_Section {
    pub fn section_type(&self) -> Ref<'_, String> {
        self.section_type.borrow()
    }
}

/**
 * Should be set to `0xFFFF_FFFF` for `section_type != "DATA"`
 */
impl Grub2Font_Section {
    pub fn len_body(&self) -> Ref<'_, u32> {
        self.len_body.borrow()
    }
}
impl Grub2Font_Section {
    pub fn body(&self) -> Ref<'_, Option<Grub2Font_Section_Body>> {
        self.body.borrow()
    }
}
impl Grub2Font_Section {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Grub2Font_Section {
    pub fn section_type_raw(&self) -> Ref<'_, Vec<u8>> {
        self.section_type_raw.borrow()
    }
}
impl Grub2Font_Section {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_SlanSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    font_slant: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_SlanSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.font_slant.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_SlanSection {
}
impl Grub2Font_SlanSection {
    pub fn font_slant(&self) -> Ref<'_, String> {
        self.font_slant.borrow()
    }
}
impl Grub2Font_SlanSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grub2Font_WeigSection {
    pub(crate) _root: SharedType<Grub2Font>,
    pub(crate) _parent: SharedType<Grub2Font_Section>,
    pub(crate) _self_shared: SharedType<Self>,
    font_weight: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Grub2Font_WeigSection {
    type Root = Grub2Font;
    type Parent = Grub2Font_Section;

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
        *self_rc.font_weight.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Grub2Font_WeigSection {
}
impl Grub2Font_WeigSection {
    pub fn font_weight(&self) -> Ref<'_, String> {
        self.font_weight.borrow()
    }
}
impl Grub2Font_WeigSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
