// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * ISO9660 is standard filesystem used on read-only optical discs
 * (mostly CD-ROM). The standard was based on earlier High Sierra
 * Format (HSF), proposed for CD-ROMs in 1985, and, after several
 * revisions, it was accepted as ISO9960:1998.
 *
 * The format emphasizes portability (thus having pretty minimal
 * features and very conservative file names standards) and sequential
 * access (which favors disc devices with relatively slow rotation
 * speed).
 */

#[derive(Default, Debug, Clone)]
pub struct Iso9660 {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_primary_vol_desc: Cell<bool>,
    primary_vol_desc: RefCell<OptRc<Iso9660_VolDesc>>,
    f_sector_size: Cell<bool>,
    sector_size: RefCell<i32>,
}
impl KStruct for Iso9660 {
    type Root = Iso9660;
    type Parent = Iso9660;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn primary_vol_desc(
        &self
    ) -> KResult<Ref<'_, OptRc<Iso9660_VolDesc>>> {
        let _io = self._io.borrow();
        if self.f_primary_vol_desc.get() {
            return Ok(self.primary_vol_desc.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from((16_i32).saturating_mul(*self.sector_size()?))?)?;
        let t = Self::read_into::<_, Iso9660_VolDesc>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.primary_vol_desc.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.primary_vol_desc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sector_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sector_size.get() {
            return Ok(self.sector_size.borrow());
        }
        self.f_sector_size.set(true);
        *self.sector_size.borrow_mut() = (2048).try_into()?;
        Ok(self.sector_size.borrow())
    }
}
impl Iso9660 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_Datetime {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_DirEntryBody>,
    pub(crate) _self_shared: SharedType<Self>,
    year: RefCell<u8>,
    month: RefCell<u8>,
    day: RefCell<u8>,
    hour: RefCell<u8>,
    minute: RefCell<u8>,
    sec: RefCell<u8>,
    timezone: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Iso9660_Datetime {
    type Root = Iso9660;
    type Parent = Iso9660_DirEntryBody;

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
        *self_rc.year.borrow_mut() = _io.read_u1()?;
        *self_rc.month.borrow_mut() = _io.read_u1()?;
        *self_rc.day.borrow_mut() = _io.read_u1()?;
        *self_rc.hour.borrow_mut() = _io.read_u1()?;
        *self_rc.minute.borrow_mut() = _io.read_u1()?;
        *self_rc.sec.borrow_mut() = _io.read_u1()?;
        *self_rc.timezone.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_Datetime {
}
impl Iso9660_Datetime {
    pub fn year(&self) -> Ref<'_, u8> {
        self.year.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn month(&self) -> Ref<'_, u8> {
        self.month.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn day(&self) -> Ref<'_, u8> {
        self.day.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn hour(&self) -> Ref<'_, u8> {
        self.hour.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn minute(&self) -> Ref<'_, u8> {
        self.minute.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn sec(&self) -> Ref<'_, u8> {
        self.sec.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn timezone(&self) -> Ref<'_, u8> {
        self.timezone.borrow()
    }
}
impl Iso9660_Datetime {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://wiki.osdev.org/ISO_9660#Date.2Ftime_format> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Iso9660_DecDatetime {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_VolDescPrimary>,
    pub(crate) _self_shared: SharedType<Self>,
    year: RefCell<String>,
    month: RefCell<String>,
    day: RefCell<String>,
    hour: RefCell<String>,
    minute: RefCell<String>,
    sec: RefCell<String>,
    sec_hundreds: RefCell<String>,
    timezone: RefCell<u8>,
    _io: RefCell<BytesReader>,
    year_raw: RefCell<Vec<u8>>,
    month_raw: RefCell<Vec<u8>>,
    day_raw: RefCell<Vec<u8>>,
    hour_raw: RefCell<Vec<u8>>,
    minute_raw: RefCell<Vec<u8>>,
    sec_raw: RefCell<Vec<u8>>,
    sec_hundreds_raw: RefCell<Vec<u8>>,
}
impl KStruct for Iso9660_DecDatetime {
    type Root = Iso9660;
    type Parent = Iso9660_VolDescPrimary;

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
        *self_rc.year.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        *self_rc.month.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        *self_rc.day.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        *self_rc.hour.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        *self_rc.minute.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        *self_rc.sec.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        *self_rc.sec_hundreds.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        *self_rc.timezone.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_DecDatetime {
}
impl Iso9660_DecDatetime {
    pub fn year(&self) -> Ref<'_, String> {
        self.year.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn month(&self) -> Ref<'_, String> {
        self.month.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn day(&self) -> Ref<'_, String> {
        self.day.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn hour(&self) -> Ref<'_, String> {
        self.hour.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn minute(&self) -> Ref<'_, String> {
        self.minute.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn sec(&self) -> Ref<'_, String> {
        self.sec.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn sec_hundreds(&self) -> Ref<'_, String> {
        self.sec_hundreds.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn timezone(&self) -> Ref<'_, u8> {
        self.timezone.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn year_raw(&self) -> Ref<'_, Vec<u8>> {
        self.year_raw.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn month_raw(&self) -> Ref<'_, Vec<u8>> {
        self.month_raw.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn day_raw(&self) -> Ref<'_, Vec<u8>> {
        self.day_raw.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn hour_raw(&self) -> Ref<'_, Vec<u8>> {
        self.hour_raw.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn minute_raw(&self) -> Ref<'_, Vec<u8>> {
        self.minute_raw.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn sec_raw(&self) -> Ref<'_, Vec<u8>> {
        self.sec_raw.borrow()
    }
}
impl Iso9660_DecDatetime {
    pub fn sec_hundreds_raw(&self) -> Ref<'_, Vec<u8>> {
        self.sec_hundreds_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_DirEntries {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_DirEntryBody>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<Iso9660_DirEntry>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Iso9660_DirEntries {
    type Root = Iso9660;
    type Parent = Iso9660_DirEntryBody;

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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, Iso9660_DirEntry>(&*_io, Some(self_rc._root.clone()), None)?.into();
                self_rc.entries.borrow_mut().push(t);
                let _t_entries = self_rc.entries.borrow();
                let Some(_tmpa) = _t_entries.last() else { break; };
                _i = _i.saturating_add(1);
                if ((to_i128(*_tmpa.len())) == (to_i128(0))) { break; }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_DirEntries {
}
impl Iso9660_DirEntries {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<Iso9660_DirEntry>>> {
        self.entries.borrow()
    }
}
impl Iso9660_DirEntries {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_DirEntry {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<u8>,
    body: RefCell<OptRc<Iso9660_DirEntryBody>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
impl KStruct for Iso9660_DirEntry {
    type Root = Iso9660;
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
        *self_rc.len.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.len())) > (to_i128(0))) {
            let _raw_body = _io.read_bytes(usize::try_from((i32::from(*self_rc.len())).saturating_sub(1_i32))?)?;
            *self_rc.body_raw.borrow_mut() = _raw_body.clone();
            let _io_body = BytesReader::from(_raw_body);
            let t = Self::read_into::<BytesReader, Iso9660_DirEntryBody>(&_io_body, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.body.borrow_mut() = t;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_DirEntry {
}
impl Iso9660_DirEntry {
    pub fn len(&self) -> Ref<'_, u8> {
        self.len.borrow()
    }
}
impl Iso9660_DirEntry {
    pub fn body(&self) -> Ref<'_, OptRc<Iso9660_DirEntryBody>> {
        self.body.borrow()
    }
}
impl Iso9660_DirEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Iso9660_DirEntry {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_DirEntryBody {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_DirEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    len_ext_attr_rec: RefCell<u8>,
    lba_extent: RefCell<OptRc<Iso9660_U4bi>>,
    size_extent: RefCell<OptRc<Iso9660_U4bi>>,
    datetime: RefCell<OptRc<Iso9660_Datetime>>,
    file_flags: RefCell<u8>,
    file_unit_size: RefCell<u8>,
    interleave_gap_size: RefCell<u8>,
    vol_seq_num: RefCell<OptRc<Iso9660_U2bi>>,
    len_file_name: RefCell<u8>,
    file_name: RefCell<String>,
    padding: RefCell<u8>,
    rest: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    file_name_raw: RefCell<Vec<u8>>,
    extent_as_dir_raw: RefCell<Vec<u8>>,
    f_extent_as_dir: Cell<bool>,
    extent_as_dir: RefCell<OptRc<Iso9660_DirEntries>>,
    f_extent_as_file: Cell<bool>,
    extent_as_file: RefCell<Vec<u8>>,
}
impl KStruct for Iso9660_DirEntryBody {
    type Root = Iso9660;
    type Parent = Iso9660_DirEntry;

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
        *self_rc.len_ext_attr_rec.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, Iso9660_U4bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.lba_extent.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_U4bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.size_extent.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_Datetime>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.datetime.borrow_mut() = t;
        *self_rc.file_flags.borrow_mut() = _io.read_u1()?;
        *self_rc.file_unit_size.borrow_mut() = _io.read_u1()?;
        *self_rc.interleave_gap_size.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, Iso9660_U2bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.vol_seq_num.borrow_mut() = t;
        *self_rc.len_file_name.borrow_mut() = _io.read_u1()?;
        *self_rc.file_name.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_file_name()))?, "UTF-8")?;
        if (i32::from(*self_rc.len_file_name())).checked_rem(2_i32).ok_or(KError::CastError)? == 0 {
            *self_rc.padding.borrow_mut() = _io.read_u1()?;
        }
        *self_rc.rest.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_DirEntryBody {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn extent_as_dir(
        &self
    ) -> KResult<Ref<'_, OptRc<Iso9660_DirEntries>>> {
        let _io = self._io.borrow();
        if self.f_extent_as_dir.get() {
            return Ok(self.extent_as_dir.borrow());
        }
        if ((i32::from(*self.file_flags())) & (2_i32)) != 0 {
            let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
            let _pos = io.pos();
            io.seek(usize::try_from((*self.lba_extent().le()).saturating_mul(u32::try_from(*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.sector_size()?)?))?)?;
            *self.extent_as_dir_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self.size_extent().le())?)?.into();
            let extent_as_dir_raw = self.extent_as_dir_raw.borrow();
            let _t_extent_as_dir_raw_io = BytesReader::from(extent_as_dir_raw.clone());
            let t = Self::read_into::<BytesReader, Iso9660_DirEntries>(&_t_extent_as_dir_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
            *self.extent_as_dir.borrow_mut() = t;
            io.seek(_pos)?;
        }
        Ok(self.extent_as_dir.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn extent_as_file(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_extent_as_file.get() {
            return Ok(self.extent_as_file.borrow());
        }
        self.f_extent_as_file.set(true);
        if ((i32::from(*self.file_flags())) & (2_i32)) == 0 {
            let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
            let _pos = io.pos();
            io.seek(usize::try_from((*self.lba_extent().le()).saturating_mul(u32::try_from(*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.sector_size()?)?))?)?;
            *self.extent_as_file.borrow_mut() = io.read_bytes(usize::try_from(*self.size_extent().le())?)?;
            io.seek(_pos)?;
        }
        Ok(self.extent_as_file.borrow())
    }
}
impl Iso9660_DirEntryBody {
    pub fn len_ext_attr_rec(&self) -> Ref<'_, u8> {
        self.len_ext_attr_rec.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn lba_extent(&self) -> Ref<'_, OptRc<Iso9660_U4bi>> {
        self.lba_extent.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn size_extent(&self) -> Ref<'_, OptRc<Iso9660_U4bi>> {
        self.size_extent.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn datetime(&self) -> Ref<'_, OptRc<Iso9660_Datetime>> {
        self.datetime.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn file_flags(&self) -> Ref<'_, u8> {
        self.file_flags.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn file_unit_size(&self) -> Ref<'_, u8> {
        self.file_unit_size.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn interleave_gap_size(&self) -> Ref<'_, u8> {
        self.interleave_gap_size.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn vol_seq_num(&self) -> Ref<'_, OptRc<Iso9660_U2bi>> {
        self.vol_seq_num.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn len_file_name(&self) -> Ref<'_, u8> {
        self.len_file_name.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn file_name(&self) -> Ref<'_, String> {
        self.file_name.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn padding(&self) -> Ref<'_, u8> {
        self.padding.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn rest(&self) -> Ref<'_, Vec<u8>> {
        self.rest.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn file_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.file_name_raw.borrow()
    }
}
impl Iso9660_DirEntryBody {
    pub fn extent_as_dir_raw(&self) -> Ref<'_, Vec<u8>> {
        self.extent_as_dir_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_PathTableEntryLe {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_PathTableLe>,
    pub(crate) _self_shared: SharedType<Self>,
    len_dir_name: RefCell<u8>,
    len_ext_attr_rec: RefCell<u8>,
    lba_extent: RefCell<u32>,
    parent_dir_idx: RefCell<u16>,
    dir_name: RefCell<String>,
    padding: RefCell<u8>,
    _io: RefCell<BytesReader>,
    dir_name_raw: RefCell<Vec<u8>>,
}
impl KStruct for Iso9660_PathTableEntryLe {
    type Root = Iso9660;
    type Parent = Iso9660_PathTableLe;

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
        *self_rc.len_dir_name.borrow_mut() = _io.read_u1()?;
        *self_rc.len_ext_attr_rec.borrow_mut() = _io.read_u1()?;
        *self_rc.lba_extent.borrow_mut() = _io.read_u4le()?;
        *self_rc.parent_dir_idx.borrow_mut() = _io.read_u2le()?;
        *self_rc.dir_name.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_dir_name()))?, "UTF-8")?;
        if (i32::from(*self_rc.len_dir_name())).checked_rem(2_i32).ok_or(KError::CastError)? == 1 {
            *self_rc.padding.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_PathTableEntryLe {
}
impl Iso9660_PathTableEntryLe {
    pub fn len_dir_name(&self) -> Ref<'_, u8> {
        self.len_dir_name.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn len_ext_attr_rec(&self) -> Ref<'_, u8> {
        self.len_ext_attr_rec.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn lba_extent(&self) -> Ref<'_, u32> {
        self.lba_extent.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn parent_dir_idx(&self) -> Ref<'_, u16> {
        self.parent_dir_idx.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn dir_name(&self) -> Ref<'_, String> {
        self.dir_name.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn padding(&self) -> Ref<'_, u8> {
        self.padding.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Iso9660_PathTableEntryLe {
    pub fn dir_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.dir_name_raw.borrow()
    }
}

/**
 * \sa <https://wiki.osdev.org/ISO_9660#The_Path_Table> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Iso9660_PathTableLe {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_VolDescPrimary>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<Iso9660_PathTableEntryLe>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Iso9660_PathTableLe {
    type Root = Iso9660;
    type Parent = Iso9660_VolDescPrimary;

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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Iso9660_PathTableEntryLe>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_PathTableLe {
}
impl Iso9660_PathTableLe {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<Iso9660_PathTableEntryLe>>> {
        self.entries.borrow()
    }
}
impl Iso9660_PathTableLe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_U2bi {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    le: RefCell<u16>,
    be: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Iso9660_U2bi {
    type Root = Iso9660;
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
        *self_rc.le.borrow_mut() = _io.read_u2le()?;
        *self_rc.be.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_U2bi {
}
impl Iso9660_U2bi {
    pub fn le(&self) -> Ref<'_, u16> {
        self.le.borrow()
    }
}
impl Iso9660_U2bi {
    pub fn be(&self) -> Ref<'_, u16> {
        self.be.borrow()
    }
}
impl Iso9660_U2bi {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_U4bi {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    le: RefCell<u32>,
    be: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Iso9660_U4bi {
    type Root = Iso9660;
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
        *self_rc.le.borrow_mut() = _io.read_u4le()?;
        *self_rc.be.borrow_mut() = _io.read_u4be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_U4bi {
}
impl Iso9660_U4bi {
    pub fn le(&self) -> Ref<'_, u32> {
        self.le.borrow()
    }
}
impl Iso9660_U4bi {
    pub fn be(&self) -> Ref<'_, u32> {
        self.be.borrow()
    }
}
impl Iso9660_U4bi {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_VolDesc {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660>,
    pub(crate) _self_shared: SharedType<Self>,
    r#type: RefCell<u8>,
    magic: RefCell<Vec<u8>>,
    version: RefCell<u8>,
    vol_desc_boot_record: RefCell<OptRc<Iso9660_VolDescBootRecord>>,
    vol_desc_primary: RefCell<OptRc<Iso9660_VolDescPrimary>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Iso9660_VolDesc {
    type Root = Iso9660;
    type Parent = Iso9660;

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
        *self_rc.r#type.borrow_mut() = _io.read_u1()?;
        *self_rc.magic.borrow_mut() = _io.read_bytes(5_usize)?;
        if !(*self_rc.magic() == vec![0x43u8, 0x44u8, 0x30u8, 0x30u8, 0x31u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vol_desc/seq/1".to_string() }));
        }
        *self_rc.version.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.r#type())) == (to_i128(0))) {
            let t = Self::read_into::<_, Iso9660_VolDescBootRecord>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.vol_desc_boot_record.borrow_mut() = t;
        }
        if ((to_i128(*self_rc.r#type())) == (to_i128(1))) {
            let t = Self::read_into::<_, Iso9660_VolDescPrimary>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.vol_desc_primary.borrow_mut() = t;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_VolDesc {
}
impl Iso9660_VolDesc {
    pub fn r#type(&self) -> Ref<'_, u8> {
        self.r#type.borrow()
    }
}
impl Iso9660_VolDesc {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Iso9660_VolDesc {
    pub fn version(&self) -> Ref<'_, u8> {
        self.version.borrow()
    }
}
impl Iso9660_VolDesc {
    pub fn vol_desc_boot_record(&self) -> Ref<'_, OptRc<Iso9660_VolDescBootRecord>> {
        self.vol_desc_boot_record.borrow()
    }
}
impl Iso9660_VolDesc {
    pub fn vol_desc_primary(&self) -> Ref<'_, OptRc<Iso9660_VolDescPrimary>> {
        self.vol_desc_primary.borrow()
    }
}
impl Iso9660_VolDesc {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Iso9660_VolDescBootRecord {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_VolDesc>,
    pub(crate) _self_shared: SharedType<Self>,
    boot_system_id: RefCell<String>,
    boot_id: RefCell<String>,
    _io: RefCell<BytesReader>,
    boot_system_id_raw: RefCell<Vec<u8>>,
    boot_id_raw: RefCell<Vec<u8>>,
}
impl KStruct for Iso9660_VolDescBootRecord {
    type Root = Iso9660;
    type Parent = Iso9660_VolDesc;

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
        *self_rc.boot_system_id.borrow_mut() = bytes_to_str(&_io.read_bytes(32_usize)?, "UTF-8")?;
        *self_rc.boot_id.borrow_mut() = bytes_to_str(&_io.read_bytes(32_usize)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_VolDescBootRecord {
}
impl Iso9660_VolDescBootRecord {
    pub fn boot_system_id(&self) -> Ref<'_, String> {
        self.boot_system_id.borrow()
    }
}
impl Iso9660_VolDescBootRecord {
    pub fn boot_id(&self) -> Ref<'_, String> {
        self.boot_id.borrow()
    }
}
impl Iso9660_VolDescBootRecord {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Iso9660_VolDescBootRecord {
    pub fn boot_system_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.boot_system_id_raw.borrow()
    }
}
impl Iso9660_VolDescBootRecord {
    pub fn boot_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.boot_id_raw.borrow()
    }
}

/**
 * \sa <https://wiki.osdev.org/ISO_9660#The_Primary_Volume_Descriptor> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Iso9660_VolDescPrimary {
    pub(crate) _root: SharedType<Iso9660>,
    pub(crate) _parent: SharedType<Iso9660_VolDesc>,
    pub(crate) _self_shared: SharedType<Self>,
    unused1: RefCell<Vec<u8>>,
    system_id: RefCell<String>,
    volume_id: RefCell<String>,
    unused2: RefCell<Vec<u8>>,
    vol_space_size: RefCell<OptRc<Iso9660_U4bi>>,
    unused3: RefCell<Vec<u8>>,
    vol_set_size: RefCell<OptRc<Iso9660_U2bi>>,
    vol_seq_num: RefCell<OptRc<Iso9660_U2bi>>,
    logical_block_size: RefCell<OptRc<Iso9660_U2bi>>,
    path_table_size: RefCell<OptRc<Iso9660_U4bi>>,
    lba_path_table_le: RefCell<u32>,
    lba_opt_path_table_le: RefCell<u32>,
    lba_path_table_be: RefCell<u32>,
    lba_opt_path_table_be: RefCell<u32>,
    root_dir: RefCell<OptRc<Iso9660_DirEntry>>,
    vol_set_id: RefCell<String>,
    publisher_id: RefCell<String>,
    data_preparer_id: RefCell<String>,
    application_id: RefCell<String>,
    copyright_file_id: RefCell<String>,
    abstract_file_id: RefCell<String>,
    bibliographic_file_id: RefCell<String>,
    vol_create_datetime: RefCell<OptRc<Iso9660_DecDatetime>>,
    vol_mod_datetime: RefCell<OptRc<Iso9660_DecDatetime>>,
    vol_expire_datetime: RefCell<OptRc<Iso9660_DecDatetime>>,
    vol_effective_datetime: RefCell<OptRc<Iso9660_DecDatetime>>,
    file_structure_version: RefCell<u8>,
    unused4: RefCell<u8>,
    application_area: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    system_id_raw: RefCell<Vec<u8>>,
    volume_id_raw: RefCell<Vec<u8>>,
    root_dir_raw: RefCell<Vec<u8>>,
    vol_set_id_raw: RefCell<Vec<u8>>,
    publisher_id_raw: RefCell<Vec<u8>>,
    data_preparer_id_raw: RefCell<Vec<u8>>,
    application_id_raw: RefCell<Vec<u8>>,
    copyright_file_id_raw: RefCell<Vec<u8>>,
    abstract_file_id_raw: RefCell<Vec<u8>>,
    bibliographic_file_id_raw: RefCell<Vec<u8>>,
    application_area_raw: RefCell<Vec<u8>>,
    path_table_raw: RefCell<Vec<u8>>,
    f_path_table: Cell<bool>,
    path_table: RefCell<OptRc<Iso9660_PathTableLe>>,
}
impl KStruct for Iso9660_VolDescPrimary {
    type Root = Iso9660;
    type Parent = Iso9660_VolDesc;

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
        *self_rc.unused1.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.unused1() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vol_desc_primary/seq/0".to_string() }));
        }
        *self_rc.system_id.borrow_mut() = bytes_to_str(&_io.read_bytes(32_usize)?, "UTF-8")?;
        *self_rc.volume_id.borrow_mut() = bytes_to_str(&_io.read_bytes(32_usize)?, "UTF-8")?;
        *self_rc.unused2.borrow_mut() = _io.read_bytes(8_usize)?;
        if !(*self_rc.unused2() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vol_desc_primary/seq/3".to_string() }));
        }
        let t = Self::read_into::<_, Iso9660_U4bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.vol_space_size.borrow_mut() = t;
        *self_rc.unused3.borrow_mut() = _io.read_bytes(32_usize)?;
        if !(*self_rc.unused3() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vol_desc_primary/seq/5".to_string() }));
        }
        let t = Self::read_into::<_, Iso9660_U2bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.vol_set_size.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_U2bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.vol_seq_num.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_U2bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.logical_block_size.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_U4bi>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.path_table_size.borrow_mut() = t;
        *self_rc.lba_path_table_le.borrow_mut() = _io.read_u4le()?;
        *self_rc.lba_opt_path_table_le.borrow_mut() = _io.read_u4le()?;
        *self_rc.lba_path_table_be.borrow_mut() = _io.read_u4be()?;
        *self_rc.lba_opt_path_table_be.borrow_mut() = _io.read_u4be()?;
        let _raw_root_dir = _io.read_bytes(34_usize)?;
        *self_rc.root_dir_raw.borrow_mut() = _raw_root_dir.clone();
        let _io_root_dir = BytesReader::from(_raw_root_dir);
        let t = Self::read_into::<BytesReader, Iso9660_DirEntry>(&_io_root_dir, Some(self_rc._root.clone()), None)?.into();
        *self_rc.root_dir.borrow_mut() = t;
        *self_rc.vol_set_id.borrow_mut() = bytes_to_str(&_io.read_bytes(128_usize)?, "UTF-8")?;
        *self_rc.publisher_id.borrow_mut() = bytes_to_str(&_io.read_bytes(128_usize)?, "UTF-8")?;
        *self_rc.data_preparer_id.borrow_mut() = bytes_to_str(&_io.read_bytes(128_usize)?, "UTF-8")?;
        *self_rc.application_id.borrow_mut() = bytes_to_str(&_io.read_bytes(128_usize)?, "UTF-8")?;
        *self_rc.copyright_file_id.borrow_mut() = bytes_to_str(&_io.read_bytes(38_usize)?, "UTF-8")?;
        *self_rc.abstract_file_id.borrow_mut() = bytes_to_str(&_io.read_bytes(36_usize)?, "UTF-8")?;
        *self_rc.bibliographic_file_id.borrow_mut() = bytes_to_str(&_io.read_bytes(37_usize)?, "UTF-8")?;
        let t = Self::read_into::<_, Iso9660_DecDatetime>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.vol_create_datetime.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_DecDatetime>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.vol_mod_datetime.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_DecDatetime>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.vol_expire_datetime.borrow_mut() = t;
        let t = Self::read_into::<_, Iso9660_DecDatetime>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.vol_effective_datetime.borrow_mut() = t;
        *self_rc.file_structure_version.borrow_mut() = _io.read_u1()?;
        *self_rc.unused4.borrow_mut() = _io.read_u1()?;
        *self_rc.application_area.borrow_mut() = _io.read_bytes(512_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Iso9660_VolDescPrimary {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn path_table(
        &self
    ) -> KResult<Ref<'_, OptRc<Iso9660_PathTableLe>>> {
        let _io = self._io.borrow();
        if self.f_path_table.get() {
            return Ok(self.path_table.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from((*self.lba_path_table_le()).saturating_mul(u32::try_from(*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.sector_size()?)?))?)?;
        *self.path_table_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self.path_table_size().le())?)?.into();
        let path_table_raw = self.path_table_raw.borrow();
        let _t_path_table_raw_io = BytesReader::from(path_table_raw.clone());
        let t = Self::read_into::<BytesReader, Iso9660_PathTableLe>(&_t_path_table_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.path_table.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.path_table.borrow())
    }
}
impl Iso9660_VolDescPrimary {
    pub fn unused1(&self) -> Ref<'_, Vec<u8>> {
        self.unused1.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn system_id(&self) -> Ref<'_, String> {
        self.system_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn volume_id(&self) -> Ref<'_, String> {
        self.volume_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn unused2(&self) -> Ref<'_, Vec<u8>> {
        self.unused2.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_space_size(&self) -> Ref<'_, OptRc<Iso9660_U4bi>> {
        self.vol_space_size.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn unused3(&self) -> Ref<'_, Vec<u8>> {
        self.unused3.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_set_size(&self) -> Ref<'_, OptRc<Iso9660_U2bi>> {
        self.vol_set_size.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_seq_num(&self) -> Ref<'_, OptRc<Iso9660_U2bi>> {
        self.vol_seq_num.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn logical_block_size(&self) -> Ref<'_, OptRc<Iso9660_U2bi>> {
        self.logical_block_size.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn path_table_size(&self) -> Ref<'_, OptRc<Iso9660_U4bi>> {
        self.path_table_size.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn lba_path_table_le(&self) -> Ref<'_, u32> {
        self.lba_path_table_le.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn lba_opt_path_table_le(&self) -> Ref<'_, u32> {
        self.lba_opt_path_table_le.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn lba_path_table_be(&self) -> Ref<'_, u32> {
        self.lba_path_table_be.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn lba_opt_path_table_be(&self) -> Ref<'_, u32> {
        self.lba_opt_path_table_be.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn root_dir(&self) -> Ref<'_, OptRc<Iso9660_DirEntry>> {
        self.root_dir.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_set_id(&self) -> Ref<'_, String> {
        self.vol_set_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn publisher_id(&self) -> Ref<'_, String> {
        self.publisher_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn data_preparer_id(&self) -> Ref<'_, String> {
        self.data_preparer_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn application_id(&self) -> Ref<'_, String> {
        self.application_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn copyright_file_id(&self) -> Ref<'_, String> {
        self.copyright_file_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn abstract_file_id(&self) -> Ref<'_, String> {
        self.abstract_file_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn bibliographic_file_id(&self) -> Ref<'_, String> {
        self.bibliographic_file_id.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_create_datetime(&self) -> Ref<'_, OptRc<Iso9660_DecDatetime>> {
        self.vol_create_datetime.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_mod_datetime(&self) -> Ref<'_, OptRc<Iso9660_DecDatetime>> {
        self.vol_mod_datetime.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_expire_datetime(&self) -> Ref<'_, OptRc<Iso9660_DecDatetime>> {
        self.vol_expire_datetime.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_effective_datetime(&self) -> Ref<'_, OptRc<Iso9660_DecDatetime>> {
        self.vol_effective_datetime.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn file_structure_version(&self) -> Ref<'_, u8> {
        self.file_structure_version.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn unused4(&self) -> Ref<'_, u8> {
        self.unused4.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn application_area(&self) -> Ref<'_, Vec<u8>> {
        self.application_area.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn system_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.system_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn volume_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.volume_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn root_dir_raw(&self) -> Ref<'_, Vec<u8>> {
        self.root_dir_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn vol_set_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.vol_set_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn publisher_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.publisher_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn data_preparer_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_preparer_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn application_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.application_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn copyright_file_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.copyright_file_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn abstract_file_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.abstract_file_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn bibliographic_file_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bibliographic_file_id_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn application_area_raw(&self) -> Ref<'_, Vec<u8>> {
        self.application_area_raw.borrow()
    }
}
impl Iso9660_VolDescPrimary {
    pub fn path_table_raw(&self) -> Ref<'_, Vec<u8>> {
        self.path_table_raw.borrow()
    }
}
