// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct DoomWad {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<String>,
    num_index_entries: RefCell<i32>,
    index_offset: RefCell<i32>,
    _io: RefCell<BytesReader>,
    f_index: Cell<bool>,
    index: RefCell<Vec<OptRc<DoomWad_IndexEntry>>>,
}
impl KStruct for DoomWad {
    type Root = DoomWad;
    type Parent = DoomWad;

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
        *self_rc.magic.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        *self_rc.num_index_entries.borrow_mut() = _io.read_s4le()?;
        *self_rc.index_offset.borrow_mut() = _io.read_s4le()?;
        Ok(())
    }
}
impl DoomWad {
    pub fn index(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<DoomWad_IndexEntry>>>> {
        let _io = self._io.borrow();
        if self.f_index.get() {
            return Ok(self.index.borrow());
        }
        self.f_index.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.index_offset())?)?;
        *self.index.borrow_mut() = Vec::new();
        let l_index = usize::try_from(*self.num_index_entries())?;
        for _i in 0_usize..l_index {
            let t = Self::read_into::<_, DoomWad_IndexEntry>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
            self.index.borrow_mut().push(t);
        }
        _io.seek(_pos)?;
        Ok(self.index.borrow())
    }
}
impl DoomWad {
    pub fn magic(&self) -> Ref<'_, String> {
        self.magic.borrow()
    }
}

/**
 * Number of entries in the lump index
 */
impl DoomWad {
    pub fn num_index_entries(&self) -> Ref<'_, i32> {
        self.num_index_entries.borrow()
    }
}

/**
 * Offset to the start of the index
 */
impl DoomWad {
    pub fn index_offset(&self) -> Ref<'_, i32> {
        self.index_offset.borrow()
    }
}
impl DoomWad {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Blockmap {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    origin_x: RefCell<i16>,
    origin_y: RefCell<i16>,
    num_cols: RefCell<i16>,
    num_rows: RefCell<i16>,
    linedefs_in_block: RefCell<Vec<OptRc<DoomWad_Blockmap_Blocklist>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Blockmap {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
        *self_rc.origin_x.borrow_mut() = _io.read_s2le()?;
        *self_rc.origin_y.borrow_mut() = _io.read_s2le()?;
        *self_rc.num_cols.borrow_mut() = _io.read_s2le()?;
        *self_rc.num_rows.borrow_mut() = _io.read_s2le()?;
        *self_rc.linedefs_in_block.borrow_mut() = Vec::new();
        let l_linedefs_in_block = usize::try_from((*self_rc.num_cols()).saturating_mul(*self_rc.num_rows()))?;
        for _i in 0_usize..l_linedefs_in_block {
            let t = Self::read_into::<_, DoomWad_Blockmap_Blocklist>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.linedefs_in_block.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl DoomWad_Blockmap {
}

/**
 * Grid origin, X coord
 */
impl DoomWad_Blockmap {
    pub fn origin_x(&self) -> Ref<'_, i16> {
        self.origin_x.borrow()
    }
}

/**
 * Grid origin, Y coord
 */
impl DoomWad_Blockmap {
    pub fn origin_y(&self) -> Ref<'_, i16> {
        self.origin_y.borrow()
    }
}

/**
 * Number of columns
 */
impl DoomWad_Blockmap {
    pub fn num_cols(&self) -> Ref<'_, i16> {
        self.num_cols.borrow()
    }
}

/**
 * Number of rows
 */
impl DoomWad_Blockmap {
    pub fn num_rows(&self) -> Ref<'_, i16> {
        self.num_rows.borrow()
    }
}

/**
 * Lists of linedefs for every block
 */
impl DoomWad_Blockmap {
    pub fn linedefs_in_block(&self) -> Ref<'_, Vec<OptRc<DoomWad_Blockmap_Blocklist>>> {
        self.linedefs_in_block.borrow()
    }
}
impl DoomWad_Blockmap {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Blockmap_Blocklist {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Blockmap>,
    pub(crate) _self_shared: SharedType<Self>,
    offset: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_linedefs: Cell<bool>,
    linedefs: RefCell<Vec<i16>>,
}
impl KStruct for DoomWad_Blockmap_Blocklist {
    type Root = DoomWad;
    type Parent = DoomWad_Blockmap;

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
        *self_rc.offset.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl DoomWad_Blockmap_Blocklist {

    /**
     * List of linedefs found in this block
     */
    pub fn linedefs(
        &self
    ) -> KResult<Ref<'_, Vec<i16>>> {
        let _io = self._io.borrow();
        if self.f_linedefs.get() {
            return Ok(self.linedefs.borrow());
        }
        self.f_linedefs.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((i32::from(*self.offset())).saturating_mul(2_i32))?)?;
        *self.linedefs.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                self.linedefs.borrow_mut().push(_io.read_s2le()?);
                let _t_linedefs = self.linedefs.borrow();
                let Some(_tmpa) = _t_linedefs.last() else { break; };
                let _tmpa = *_tmpa;
                _i = _i.saturating_add(1);
                if ((to_i128(_tmpa)) == (to_i128(-(1)))) { break; }
            }
        }
        _io.seek(_pos)?;
        Ok(self.linedefs.borrow())
    }
}

/**
 * Offset to the list of linedefs
 */
impl DoomWad_Blockmap_Blocklist {
    pub fn offset(&self) -> Ref<'_, u16> {
        self.offset.borrow()
    }
}
impl DoomWad_Blockmap_Blocklist {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_IndexEntry {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad>,
    pub(crate) _self_shared: SharedType<Self>,
    offset: RefCell<i32>,
    size: RefCell<i32>,
    name: RefCell<String>,
    _io: RefCell<BytesReader>,
    contents_raw: RefCell<Vec<u8>>,
    f_contents: Cell<bool>,
    contents: RefCell<Option<DoomWad_IndexEntry_Contents>>,
}
#[derive(Debug, Clone)]
pub enum DoomWad_IndexEntry_Contents {
    DoomWad_Blockmap(OptRc<DoomWad_Blockmap>),
    DoomWad_Linedefs(OptRc<DoomWad_Linedefs>),
    DoomWad_Pnames(OptRc<DoomWad_Pnames>),
    DoomWad_Sectors(OptRc<DoomWad_Sectors>),
    DoomWad_Sidedefs(OptRc<DoomWad_Sidedefs>),
    DoomWad_Texture12(OptRc<DoomWad_Texture12>),
    DoomWad_Things(OptRc<DoomWad_Things>),
    DoomWad_Vertexes(OptRc<DoomWad_Vertexes>),
    Bytes(Vec<u8>),
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Blockmap> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Blockmap(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Blockmap, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Blockmap>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Blockmap>) -> Self {
        Self::DoomWad_Blockmap(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Linedefs> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Linedefs(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Linedefs, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Linedefs>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Linedefs>) -> Self {
        Self::DoomWad_Linedefs(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Pnames> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Pnames(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Pnames, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Pnames>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Pnames>) -> Self {
        Self::DoomWad_Pnames(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Sectors> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Sectors(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Sectors, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Sectors>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Sectors>) -> Self {
        Self::DoomWad_Sectors(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Sidedefs> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Sidedefs(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Sidedefs, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Sidedefs>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Sidedefs>) -> Self {
        Self::DoomWad_Sidedefs(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Texture12> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Texture12(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Texture12, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Texture12>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Texture12>) -> Self {
        Self::DoomWad_Texture12(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Things> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Things(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Things, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Things>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Things>) -> Self {
        Self::DoomWad_Things(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for OptRc<DoomWad_Vertexes> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::DoomWad_Vertexes(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::DoomWad_Vertexes, got {:?}", v)
    }
}
impl From<OptRc<DoomWad_Vertexes>> for DoomWad_IndexEntry_Contents {
    fn from(v: OptRc<DoomWad_Vertexes>) -> Self {
        Self::DoomWad_Vertexes(v)
    }
}
impl From<&DoomWad_IndexEntry_Contents> for Vec<u8> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &DoomWad_IndexEntry_Contents) -> Self {
        if let DoomWad_IndexEntry_Contents::Bytes(x) = v {
            return x.clone();
        }
        panic!("expected DoomWad_IndexEntry_Contents::Bytes, got {:?}", v)
    }
}
impl From<Vec<u8>> for DoomWad_IndexEntry_Contents {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for DoomWad_IndexEntry {
    type Root = DoomWad;
    type Parent = DoomWad;

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
        *self_rc.offset.borrow_mut() = _io.read_s4le()?;
        *self_rc.size.borrow_mut() = _io.read_s4le()?;
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(8_usize)?, 0), "ASCII")?;
        Ok(())
    }
}
impl DoomWad_IndexEntry {
    pub fn contents(
        &self
    ) -> KResult<Ref<'_, Option<DoomWad_IndexEntry_Contents>>> {
        let _io = self._io.borrow();
        if self.f_contents.get() {
            return Ok(self.contents.borrow());
        }
        self.f_contents.set(true);
        let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
        let _pos = io.pos();
        io.seek(usize::try_from(*self.offset())?)?;
        match self.name().as_str() {
            "BLOCKMAP" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Blockmap>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "LINEDEFS" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Linedefs>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "PNAMES" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Pnames>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "SECTORS" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Sectors>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "SIDEDEFS" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Sidedefs>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "TEXTURE1" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Texture12>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "TEXTURE2" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Texture12>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "THINGS" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Things>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            "VERTEXES" => {
                *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size())?)?.into();
                let contents_raw = self.contents_raw.borrow();
                let _t_contents_raw_io = BytesReader::from(contents_raw.clone());
                let t = Self::read_into::<BytesReader, DoomWad_Vertexes>(&_t_contents_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.contents.borrow_mut() = Some(t);
            }
            _ => {
                *self.contents.borrow_mut() = Some(io.read_bytes(usize::try_from(*self.size())?)?.into());
            }
        }
        io.seek(_pos)?;
        Ok(self.contents.borrow())
    }
}
impl DoomWad_IndexEntry {
    pub fn offset(&self) -> Ref<'_, i32> {
        self.offset.borrow()
    }
}
impl DoomWad_IndexEntry {
    pub fn size(&self) -> Ref<'_, i32> {
        self.size.borrow()
    }
}
impl DoomWad_IndexEntry {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl DoomWad_IndexEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DoomWad_IndexEntry {
    pub fn contents_raw(&self) -> Ref<'_, Vec<u8>> {
        self.contents_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Linedef {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Linedefs>,
    pub(crate) _self_shared: SharedType<Self>,
    vertex_start_idx: RefCell<u16>,
    vertex_end_idx: RefCell<u16>,
    flags: RefCell<u16>,
    line_type: RefCell<u16>,
    sector_tag: RefCell<u16>,
    sidedef_right_idx: RefCell<u16>,
    sidedef_left_idx: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Linedef {
    type Root = DoomWad;
    type Parent = DoomWad_Linedefs;

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
        *self_rc.vertex_start_idx.borrow_mut() = _io.read_u2le()?;
        *self_rc.vertex_end_idx.borrow_mut() = _io.read_u2le()?;
        *self_rc.flags.borrow_mut() = _io.read_u2le()?;
        *self_rc.line_type.borrow_mut() = _io.read_u2le()?;
        *self_rc.sector_tag.borrow_mut() = _io.read_u2le()?;
        *self_rc.sidedef_right_idx.borrow_mut() = _io.read_u2le()?;
        *self_rc.sidedef_left_idx.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl DoomWad_Linedef {
}
impl DoomWad_Linedef {
    pub fn vertex_start_idx(&self) -> Ref<'_, u16> {
        self.vertex_start_idx.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn vertex_end_idx(&self) -> Ref<'_, u16> {
        self.vertex_end_idx.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn flags(&self) -> Ref<'_, u16> {
        self.flags.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn line_type(&self) -> Ref<'_, u16> {
        self.line_type.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn sector_tag(&self) -> Ref<'_, u16> {
        self.sector_tag.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn sidedef_right_idx(&self) -> Ref<'_, u16> {
        self.sidedef_right_idx.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn sidedef_left_idx(&self) -> Ref<'_, u16> {
        self.sidedef_left_idx.borrow()
    }
}
impl DoomWad_Linedef {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Linedefs {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<DoomWad_Linedef>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Linedefs {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
                let t = Self::read_into::<_, DoomWad_Linedef>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl DoomWad_Linedefs {
}
impl DoomWad_Linedefs {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<DoomWad_Linedef>>> {
        self.entries.borrow()
    }
}
impl DoomWad_Linedefs {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://doom.fandom.com/wiki/PNAMES> Source
 */

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Pnames {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    num_patches: RefCell<u32>,
    names: RefCell<Vec<String>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Pnames {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
        *self_rc.num_patches.borrow_mut() = _io.read_u4le()?;
        *self_rc.names.borrow_mut() = Vec::new();
        let l_names = usize::try_from(*self_rc.num_patches())?;
        for _i in 0_usize..l_names {
            self_rc.names.borrow_mut().push(bytes_to_str(&bytes_strip_right(&_io.read_bytes(8_usize)?, 0), "ASCII")?);
        }
        Ok(())
    }
}
impl DoomWad_Pnames {
}

/**
 * Number of patches registered in this global game directory
 */
impl DoomWad_Pnames {
    pub fn num_patches(&self) -> Ref<'_, u32> {
        self.num_patches.borrow()
    }
}
impl DoomWad_Pnames {
    pub fn names(&self) -> Ref<'_, Vec<String>> {
        self.names.borrow()
    }
}
impl DoomWad_Pnames {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Sector {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Sectors>,
    pub(crate) _self_shared: SharedType<Self>,
    floor_z: RefCell<i16>,
    ceil_z: RefCell<i16>,
    floor_flat: RefCell<String>,
    ceil_flat: RefCell<String>,
    light: RefCell<i16>,
    special_type: RefCell<DoomWad_Sector_SpecialSector>,
    tag: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Sector {
    type Root = DoomWad;
    type Parent = DoomWad_Sectors;

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
        *self_rc.floor_z.borrow_mut() = _io.read_s2le()?;
        *self_rc.ceil_z.borrow_mut() = _io.read_s2le()?;
        *self_rc.floor_flat.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ASCII")?;
        *self_rc.ceil_flat.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ASCII")?;
        *self_rc.light.borrow_mut() = _io.read_s2le()?;
        *self_rc.special_type.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        *self_rc.tag.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl DoomWad_Sector {
}
impl DoomWad_Sector {
    pub fn floor_z(&self) -> Ref<'_, i16> {
        self.floor_z.borrow()
    }
}
impl DoomWad_Sector {
    pub fn ceil_z(&self) -> Ref<'_, i16> {
        self.ceil_z.borrow()
    }
}
impl DoomWad_Sector {
    pub fn floor_flat(&self) -> Ref<'_, String> {
        self.floor_flat.borrow()
    }
}
impl DoomWad_Sector {
    pub fn ceil_flat(&self) -> Ref<'_, String> {
        self.ceil_flat.borrow()
    }
}

/**
 * Light level of the sector [0..255]. Original engine uses
 * COLORMAP to render lighting, so only 32 actual levels are
 * available (i.e. 0..7, 8..15, etc).
 */
impl DoomWad_Sector {
    pub fn light(&self) -> Ref<'_, i16> {
        self.light.borrow()
    }
}
impl DoomWad_Sector {
    pub fn special_type(&self) -> Ref<'_, DoomWad_Sector_SpecialSector> {
        self.special_type.borrow()
    }
}

/**
 * Tag number. When the linedef with the same tag number is
 * activated, some effect will be triggered in this sector.
 */
impl DoomWad_Sector {
    pub fn tag(&self) -> Ref<'_, u16> {
        self.tag.borrow()
    }
}
impl DoomWad_Sector {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum DoomWad_Sector_SpecialSector {
    Normal,
    DLightFlicker,
    DLightStrobeFast,
    DLightStrobeSlow,
    DLightStrobeHurt,
    DDamageHellslime,
    DDamageNukage,
    DLightGlow,
    Secret,
    DSectorDoorCloseIn30,
    DDamageEnd,
    DLightStrobeSlowSync,
    DLightStrobeFastSync,
    DSectorDoorRaiseIn5Mins,
    DFrictionLow,
    DDamageSuperHellslime,
    DLightFireFlicker,
    DDamageLavaWimpy,
    DDamageLavaHefty,
    DScrollEastLavaDamage,
    LightPhased,
    LightSequenceStart,
    LightSequenceSpecial1,
    LightSequenceSpecial2,
    Unknown(i64),
}

impl TryFrom<i64> for DoomWad_Sector_SpecialSector {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<DoomWad_Sector_SpecialSector> {
        match flag {
            0 => Ok(DoomWad_Sector_SpecialSector::Normal),
            1 => Ok(DoomWad_Sector_SpecialSector::DLightFlicker),
            2 => Ok(DoomWad_Sector_SpecialSector::DLightStrobeFast),
            3 => Ok(DoomWad_Sector_SpecialSector::DLightStrobeSlow),
            4 => Ok(DoomWad_Sector_SpecialSector::DLightStrobeHurt),
            5 => Ok(DoomWad_Sector_SpecialSector::DDamageHellslime),
            7 => Ok(DoomWad_Sector_SpecialSector::DDamageNukage),
            8 => Ok(DoomWad_Sector_SpecialSector::DLightGlow),
            9 => Ok(DoomWad_Sector_SpecialSector::Secret),
            10 => Ok(DoomWad_Sector_SpecialSector::DSectorDoorCloseIn30),
            11 => Ok(DoomWad_Sector_SpecialSector::DDamageEnd),
            12 => Ok(DoomWad_Sector_SpecialSector::DLightStrobeSlowSync),
            13 => Ok(DoomWad_Sector_SpecialSector::DLightStrobeFastSync),
            14 => Ok(DoomWad_Sector_SpecialSector::DSectorDoorRaiseIn5Mins),
            15 => Ok(DoomWad_Sector_SpecialSector::DFrictionLow),
            16 => Ok(DoomWad_Sector_SpecialSector::DDamageSuperHellslime),
            17 => Ok(DoomWad_Sector_SpecialSector::DLightFireFlicker),
            18 => Ok(DoomWad_Sector_SpecialSector::DDamageLavaWimpy),
            19 => Ok(DoomWad_Sector_SpecialSector::DDamageLavaHefty),
            20 => Ok(DoomWad_Sector_SpecialSector::DScrollEastLavaDamage),
            21 => Ok(DoomWad_Sector_SpecialSector::LightPhased),
            22 => Ok(DoomWad_Sector_SpecialSector::LightSequenceStart),
            23 => Ok(DoomWad_Sector_SpecialSector::LightSequenceSpecial1),
            24 => Ok(DoomWad_Sector_SpecialSector::LightSequenceSpecial2),
            _ => Ok(DoomWad_Sector_SpecialSector::Unknown(flag)),
        }
    }
}

impl From<&DoomWad_Sector_SpecialSector> for i64 {
    fn from(v: &DoomWad_Sector_SpecialSector) -> Self {
        match *v {
            DoomWad_Sector_SpecialSector::Normal => 0,
            DoomWad_Sector_SpecialSector::DLightFlicker => 1,
            DoomWad_Sector_SpecialSector::DLightStrobeFast => 2,
            DoomWad_Sector_SpecialSector::DLightStrobeSlow => 3,
            DoomWad_Sector_SpecialSector::DLightStrobeHurt => 4,
            DoomWad_Sector_SpecialSector::DDamageHellslime => 5,
            DoomWad_Sector_SpecialSector::DDamageNukage => 7,
            DoomWad_Sector_SpecialSector::DLightGlow => 8,
            DoomWad_Sector_SpecialSector::Secret => 9,
            DoomWad_Sector_SpecialSector::DSectorDoorCloseIn30 => 10,
            DoomWad_Sector_SpecialSector::DDamageEnd => 11,
            DoomWad_Sector_SpecialSector::DLightStrobeSlowSync => 12,
            DoomWad_Sector_SpecialSector::DLightStrobeFastSync => 13,
            DoomWad_Sector_SpecialSector::DSectorDoorRaiseIn5Mins => 14,
            DoomWad_Sector_SpecialSector::DFrictionLow => 15,
            DoomWad_Sector_SpecialSector::DDamageSuperHellslime => 16,
            DoomWad_Sector_SpecialSector::DLightFireFlicker => 17,
            DoomWad_Sector_SpecialSector::DDamageLavaWimpy => 18,
            DoomWad_Sector_SpecialSector::DDamageLavaHefty => 19,
            DoomWad_Sector_SpecialSector::DScrollEastLavaDamage => 20,
            DoomWad_Sector_SpecialSector::LightPhased => 21,
            DoomWad_Sector_SpecialSector::LightSequenceStart => 22,
            DoomWad_Sector_SpecialSector::LightSequenceSpecial1 => 23,
            DoomWad_Sector_SpecialSector::LightSequenceSpecial2 => 24,
            DoomWad_Sector_SpecialSector::Unknown(v) => v
        }
    }
}

impl Default for DoomWad_Sector_SpecialSector {
    fn default() -> Self { DoomWad_Sector_SpecialSector::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct DoomWad_Sectors {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<DoomWad_Sector>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Sectors {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
                let t = Self::read_into::<_, DoomWad_Sector>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl DoomWad_Sectors {
}
impl DoomWad_Sectors {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<DoomWad_Sector>>> {
        self.entries.borrow()
    }
}
impl DoomWad_Sectors {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Sidedef {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Sidedefs>,
    pub(crate) _self_shared: SharedType<Self>,
    offset_x: RefCell<i16>,
    offset_y: RefCell<i16>,
    upper_texture_name: RefCell<String>,
    lower_texture_name: RefCell<String>,
    normal_texture_name: RefCell<String>,
    sector_id: RefCell<i16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Sidedef {
    type Root = DoomWad;
    type Parent = DoomWad_Sidedefs;

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
        *self_rc.offset_x.borrow_mut() = _io.read_s2le()?;
        *self_rc.offset_y.borrow_mut() = _io.read_s2le()?;
        *self_rc.upper_texture_name.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ASCII")?;
        *self_rc.lower_texture_name.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ASCII")?;
        *self_rc.normal_texture_name.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ASCII")?;
        *self_rc.sector_id.borrow_mut() = _io.read_s2le()?;
        Ok(())
    }
}
impl DoomWad_Sidedef {
}
impl DoomWad_Sidedef {
    pub fn offset_x(&self) -> Ref<'_, i16> {
        self.offset_x.borrow()
    }
}
impl DoomWad_Sidedef {
    pub fn offset_y(&self) -> Ref<'_, i16> {
        self.offset_y.borrow()
    }
}
impl DoomWad_Sidedef {
    pub fn upper_texture_name(&self) -> Ref<'_, String> {
        self.upper_texture_name.borrow()
    }
}
impl DoomWad_Sidedef {
    pub fn lower_texture_name(&self) -> Ref<'_, String> {
        self.lower_texture_name.borrow()
    }
}
impl DoomWad_Sidedef {
    pub fn normal_texture_name(&self) -> Ref<'_, String> {
        self.normal_texture_name.borrow()
    }
}
impl DoomWad_Sidedef {
    pub fn sector_id(&self) -> Ref<'_, i16> {
        self.sector_id.borrow()
    }
}
impl DoomWad_Sidedef {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Sidedefs {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<DoomWad_Sidedef>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Sidedefs {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
                let t = Self::read_into::<_, DoomWad_Sidedef>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl DoomWad_Sidedefs {
}
impl DoomWad_Sidedefs {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<DoomWad_Sidedef>>> {
        self.entries.borrow()
    }
}
impl DoomWad_Sidedefs {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Used for TEXTURE1 and TEXTURE2 lumps, which designate how to
 * combine wall patches to make wall textures. This essentially
 * provides a very simple form of image compression, allowing
 * certain elements ("patches") to be reused / recombined on
 * different textures for more variety in the game.
 * \sa <https://doom.fandom.com/wiki/TEXTURE1_and_TEXTURE2> Source
 */

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Texture12 {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    num_textures: RefCell<i32>,
    textures: RefCell<Vec<OptRc<DoomWad_Texture12_TextureIndex>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Texture12 {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
        *self_rc.num_textures.borrow_mut() = _io.read_s4le()?;
        *self_rc.textures.borrow_mut() = Vec::new();
        let l_textures = usize::try_from(*self_rc.num_textures())?;
        for _i in 0_usize..l_textures {
            let t = Self::read_into::<_, DoomWad_Texture12_TextureIndex>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.textures.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl DoomWad_Texture12 {
}

/**
 * Number of wall textures
 */
impl DoomWad_Texture12 {
    pub fn num_textures(&self) -> Ref<'_, i32> {
        self.num_textures.borrow()
    }
}
impl DoomWad_Texture12 {
    pub fn textures(&self) -> Ref<'_, Vec<OptRc<DoomWad_Texture12_TextureIndex>>> {
        self.textures.borrow()
    }
}
impl DoomWad_Texture12 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Texture12_Patch {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Texture12_TextureBody>,
    pub(crate) _self_shared: SharedType<Self>,
    origin_x: RefCell<i16>,
    origin_y: RefCell<i16>,
    patch_id: RefCell<u16>,
    step_dir: RefCell<u16>,
    colormap: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Texture12_Patch {
    type Root = DoomWad;
    type Parent = DoomWad_Texture12_TextureBody;

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
        *self_rc.origin_x.borrow_mut() = _io.read_s2le()?;
        *self_rc.origin_y.borrow_mut() = _io.read_s2le()?;
        *self_rc.patch_id.borrow_mut() = _io.read_u2le()?;
        *self_rc.step_dir.borrow_mut() = _io.read_u2le()?;
        *self_rc.colormap.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl DoomWad_Texture12_Patch {
}

/**
 * X offset to draw a patch at (pixels from left boundary of a texture)
 */
impl DoomWad_Texture12_Patch {
    pub fn origin_x(&self) -> Ref<'_, i16> {
        self.origin_x.borrow()
    }
}

/**
 * Y offset to draw a patch at (pixels from upper boundary of a texture)
 */
impl DoomWad_Texture12_Patch {
    pub fn origin_y(&self) -> Ref<'_, i16> {
        self.origin_y.borrow()
    }
}

/**
 * Identifier of a patch (as listed in PNAMES lump) to draw
 */
impl DoomWad_Texture12_Patch {
    pub fn patch_id(&self) -> Ref<'_, u16> {
        self.patch_id.borrow()
    }
}
impl DoomWad_Texture12_Patch {
    pub fn step_dir(&self) -> Ref<'_, u16> {
        self.step_dir.borrow()
    }
}
impl DoomWad_Texture12_Patch {
    pub fn colormap(&self) -> Ref<'_, u16> {
        self.colormap.borrow()
    }
}
impl DoomWad_Texture12_Patch {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Texture12_TextureBody {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Texture12_TextureIndex>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    masked: RefCell<u32>,
    width: RefCell<u16>,
    height: RefCell<u16>,
    column_directory: RefCell<u32>,
    num_patches: RefCell<u16>,
    patches: RefCell<Vec<OptRc<DoomWad_Texture12_Patch>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Texture12_TextureBody {
    type Root = DoomWad;
    type Parent = DoomWad_Texture12_TextureIndex;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(8_usize)?, 0), "ASCII")?;
        *self_rc.masked.borrow_mut() = _io.read_u4le()?;
        *self_rc.width.borrow_mut() = _io.read_u2le()?;
        *self_rc.height.borrow_mut() = _io.read_u2le()?;
        *self_rc.column_directory.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_patches.borrow_mut() = _io.read_u2le()?;
        *self_rc.patches.borrow_mut() = Vec::new();
        let l_patches = usize::try_from(*self_rc.num_patches())?;
        for _i in 0_usize..l_patches {
            let t = Self::read_into::<_, DoomWad_Texture12_Patch>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.patches.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl DoomWad_Texture12_TextureBody {
}

/**
 * Name of a texture, only `A-Z`, `0-9`, `[]_-` are valid
 */
impl DoomWad_Texture12_TextureBody {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl DoomWad_Texture12_TextureBody {
    pub fn masked(&self) -> Ref<'_, u32> {
        self.masked.borrow()
    }
}
impl DoomWad_Texture12_TextureBody {
    pub fn width(&self) -> Ref<'_, u16> {
        self.width.borrow()
    }
}
impl DoomWad_Texture12_TextureBody {
    pub fn height(&self) -> Ref<'_, u16> {
        self.height.borrow()
    }
}

/**
 * Obsolete, ignored by all DOOM versions
 */
impl DoomWad_Texture12_TextureBody {
    pub fn column_directory(&self) -> Ref<'_, u32> {
        self.column_directory.borrow()
    }
}

/**
 * Number of patches that are used in a texture
 */
impl DoomWad_Texture12_TextureBody {
    pub fn num_patches(&self) -> Ref<'_, u16> {
        self.num_patches.borrow()
    }
}
impl DoomWad_Texture12_TextureBody {
    pub fn patches(&self) -> Ref<'_, Vec<OptRc<DoomWad_Texture12_Patch>>> {
        self.patches.borrow()
    }
}
impl DoomWad_Texture12_TextureBody {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Texture12_TextureIndex {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Texture12>,
    pub(crate) _self_shared: SharedType<Self>,
    offset: RefCell<i32>,
    _io: RefCell<BytesReader>,
    f_body: Cell<bool>,
    body: RefCell<OptRc<DoomWad_Texture12_TextureBody>>,
}
impl KStruct for DoomWad_Texture12_TextureIndex {
    type Root = DoomWad;
    type Parent = DoomWad_Texture12;

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
        *self_rc.offset.borrow_mut() = _io.read_s4le()?;
        Ok(())
    }
}
impl DoomWad_Texture12_TextureIndex {
    pub fn body(
        &self
    ) -> KResult<Ref<'_, OptRc<DoomWad_Texture12_TextureBody>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.offset())?)?;
        let t = Self::read_into::<_, DoomWad_Texture12_TextureBody>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.body.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.body.borrow())
    }
}
impl DoomWad_Texture12_TextureIndex {
    pub fn offset(&self) -> Ref<'_, i32> {
        self.offset.borrow()
    }
}
impl DoomWad_Texture12_TextureIndex {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Thing {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Things>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<i16>,
    y: RefCell<i16>,
    angle: RefCell<u16>,
    r#type: RefCell<u16>,
    flags: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Thing {
    type Root = DoomWad;
    type Parent = DoomWad_Things;

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
        *self_rc.x.borrow_mut() = _io.read_s2le()?;
        *self_rc.y.borrow_mut() = _io.read_s2le()?;
        *self_rc.angle.borrow_mut() = _io.read_u2le()?;
        *self_rc.r#type.borrow_mut() = _io.read_u2le()?;
        *self_rc.flags.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl DoomWad_Thing {
}
impl DoomWad_Thing {
    pub fn x(&self) -> Ref<'_, i16> {
        self.x.borrow()
    }
}
impl DoomWad_Thing {
    pub fn y(&self) -> Ref<'_, i16> {
        self.y.borrow()
    }
}
impl DoomWad_Thing {
    pub fn angle(&self) -> Ref<'_, u16> {
        self.angle.borrow()
    }
}
impl DoomWad_Thing {
    pub fn r#type(&self) -> Ref<'_, u16> {
        self.r#type.borrow()
    }
}
impl DoomWad_Thing {
    pub fn flags(&self) -> Ref<'_, u16> {
        self.flags.borrow()
    }
}
impl DoomWad_Thing {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Things {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<DoomWad_Thing>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Things {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
                let t = Self::read_into::<_, DoomWad_Thing>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl DoomWad_Things {
}
impl DoomWad_Things {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<DoomWad_Thing>>> {
        self.entries.borrow()
    }
}
impl DoomWad_Things {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Vertex {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_Vertexes>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<i16>,
    y: RefCell<i16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Vertex {
    type Root = DoomWad;
    type Parent = DoomWad_Vertexes;

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
        *self_rc.x.borrow_mut() = _io.read_s2le()?;
        *self_rc.y.borrow_mut() = _io.read_s2le()?;
        Ok(())
    }
}
impl DoomWad_Vertex {
}
impl DoomWad_Vertex {
    pub fn x(&self) -> Ref<'_, i16> {
        self.x.borrow()
    }
}
impl DoomWad_Vertex {
    pub fn y(&self) -> Ref<'_, i16> {
        self.y.borrow()
    }
}
impl DoomWad_Vertex {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DoomWad_Vertexes {
    pub(crate) _root: SharedType<DoomWad>,
    pub(crate) _parent: SharedType<DoomWad_IndexEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<DoomWad_Vertex>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DoomWad_Vertexes {
    type Root = DoomWad;
    type Parent = DoomWad_IndexEntry;

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
                let t = Self::read_into::<_, DoomWad_Vertex>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl DoomWad_Vertexes {
}
impl DoomWad_Vertexes {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<DoomWad_Vertex>>> {
        self.entries.borrow()
    }
}
impl DoomWad_Vertexes {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
