// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Format mostly used by Google Chrome and various Android apps to store
 * resources such as translated strings, help messages and images.
 * \sa <https://web.archive.org/web/20220126211447/https://dev.chromium.org/developers/design-documents/linuxresourcesandlocalizedstrings> Source
 * \sa <https://chromium.googlesource.com/chromium/src/tools/grit/+/3c36f27/grit/format/data_pack.py> Source
 * \sa <https://chromium.googlesource.com/chromium/src/tools/grit/+/8a23eae/grit/format/data_pack.py> Source
 */

#[derive(Default, Debug, Clone)]
pub struct ChromePak {
    pub(crate) _root: SharedType<ChromePak>,
    pub(crate) _parent: SharedType<ChromePak>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u32>,
    num_resources_v4: RefCell<u32>,
    encoding: RefCell<ChromePak_Encodings>,
    v5_part: RefCell<OptRc<ChromePak_HeaderV5Part>>,
    resources: RefCell<Vec<OptRc<ChromePak_Resource>>>,
    aliases: RefCell<Vec<OptRc<ChromePak_Alias>>>,
    _io: RefCell<BytesReader>,
    f_num_aliases: Cell<bool>,
    num_aliases: RefCell<i32>,
    f_num_resources: Cell<bool>,
    num_resources: RefCell<u32>,
}
impl KStruct for ChromePak {
    type Root = ChromePak;
    type Parent = ChromePak;

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
        *self_rc.version.borrow_mut() = _io.read_u4le()?;
        if *self_rc.version() == 4 {
            *self_rc.num_resources_v4.borrow_mut() = _io.read_u4le()?;
        }
        *self_rc.encoding.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        if *self_rc.version() == 5 {
            let t = Self::read_into::<_, ChromePak_HeaderV5Part>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.v5_part.borrow_mut() = t;
        }
        *self_rc.resources.borrow_mut() = Vec::new();
        let l_resources = usize::try_from((*self_rc.num_resources()?).saturating_add(1_u32))?;
        for _i in 0_usize..l_resources {
            let f = |t : &mut ChromePak_Resource| Ok(t.set_params((_i).try_into().map_err(|_| KError::CastError)?, ((to_i128(_i)) < (to_i128(*self_rc.num_resources()?)))));
            let t = Self::read_into_with_init::<_, ChromePak_Resource>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
            self_rc.resources.borrow_mut().push(t);
        }
        *self_rc.aliases.borrow_mut() = Vec::new();
        let l_aliases = usize::try_from(*self_rc.num_aliases()?)?;
        for _i in 0_usize..l_aliases {
            let t = Self::read_into::<_, ChromePak_Alias>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.aliases.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl ChromePak {
    pub fn num_aliases(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_num_aliases.get() {
            return Ok(self.num_aliases.borrow());
        }
        self.f_num_aliases.set(true);
        *self.num_aliases.borrow_mut() = (if *self.version() == 5 { i32::from(*self.v5_part().num_aliases()) } else { 0_i32 }).try_into()?;
        Ok(self.num_aliases.borrow())
    }
    pub fn num_resources(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_num_resources.get() {
            return Ok(self.num_resources.borrow());
        }
        self.f_num_resources.set(true);
        *self.num_resources.borrow_mut() = (if *self.version() == 5 { u32::from(*self.v5_part().num_resources()) } else { *self.num_resources_v4() }).try_into()?;
        Ok(self.num_resources.borrow())
    }
}

/**
 * only versions 4 and 5 are supported
 */
impl ChromePak {
    pub fn version(&self) -> Ref<'_, u32> {
        self.version.borrow()
    }
}
impl ChromePak {
    pub fn num_resources_v4(&self) -> Ref<'_, u32> {
        self.num_resources_v4.borrow()
    }
}

/**
 * Character encoding of all text resources in the PAK file. Note that
 * the file can **always** contain binary resources, this only applies to
 * those that are supposed to hold text.
 *
 * In practice, this will probably always be `encodings::utf8` - I haven't
 * seen any organic file that would state otherwise. `UTF8` is also usually
 * hardcoded in Python scripts from the GRIT repository that generate .pak
 * files (for example
 * [`pak_util.py:79`](https://chromium.googlesource.com/chromium/src/tools/grit/+/8a23eae/pak_util.py#79)).
 */
impl ChromePak {
    pub fn encoding(&self) -> Ref<'_, ChromePak_Encodings> {
        self.encoding.borrow()
    }
}
impl ChromePak {
    pub fn v5_part(&self) -> Ref<'_, OptRc<ChromePak_HeaderV5Part>> {
        self.v5_part.borrow()
    }
}

/**
 * The length is calculated by looking at the offset of
 * the next item, so an extra entry is stored with id 0
 * and offset pointing to the end of the resources.
 */
impl ChromePak {
    pub fn resources(&self) -> Ref<'_, Vec<OptRc<ChromePak_Resource>>> {
        self.resources.borrow()
    }
}
impl ChromePak {
    pub fn aliases(&self) -> Ref<'_, Vec<OptRc<ChromePak_Alias>>> {
        self.aliases.borrow()
    }
}
impl ChromePak {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum ChromePak_Encodings {

    /**
     * file is not expected to contain any text resources
     */
    Binary,

    /**
     * all text resources are encoded in UTF-8
     */
    Utf8,

    /**
     * all text resources are encoded in UTF-16
     */
    Utf16,
    Unknown(i64),
}

impl TryFrom<i64> for ChromePak_Encodings {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ChromePak_Encodings> {
        match flag {
            0 => Ok(ChromePak_Encodings::Binary),
            1 => Ok(ChromePak_Encodings::Utf8),
            2 => Ok(ChromePak_Encodings::Utf16),
            _ => Ok(ChromePak_Encodings::Unknown(flag)),
        }
    }
}

impl From<&ChromePak_Encodings> for i64 {
    fn from(v: &ChromePak_Encodings) -> Self {
        match *v {
            ChromePak_Encodings::Binary => 0,
            ChromePak_Encodings::Utf8 => 1,
            ChromePak_Encodings::Utf16 => 2,
            ChromePak_Encodings::Unknown(v) => v
        }
    }
}

impl Default for ChromePak_Encodings {
    fn default() -> Self { ChromePak_Encodings::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct ChromePak_Alias {
    pub(crate) _root: SharedType<ChromePak>,
    pub(crate) _parent: SharedType<ChromePak>,
    pub(crate) _self_shared: SharedType<Self>,
    id: RefCell<u16>,
    resource_idx: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_resource: Cell<bool>,
    resource: RefCell<OptRc<ChromePak_Resource>>,
}
impl KStruct for ChromePak_Alias {
    type Root = ChromePak;
    type Parent = ChromePak;

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
        *self_rc.id.borrow_mut() = _io.read_u2le()?;
        *self_rc.resource_idx.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl ChromePak_Alias {
    pub fn resource(
        &self
    ) -> KResult<Ref<'_, OptRc<ChromePak_Resource>>> {
        let _io = self._io.borrow();
        if self.f_resource.get() {
            return Ok(self.resource.borrow());
        }
        *self.resource.borrow_mut() = self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.resources().get(usize::try_from(*self.resource_idx())?).ok_or(KError::CastError)?.clone();
        Ok(self.resource.borrow())
    }
}
impl ChromePak_Alias {
    pub fn id(&self) -> Ref<'_, u16> {
        self.id.borrow()
    }
}
impl ChromePak_Alias {
    pub fn resource_idx(&self) -> Ref<'_, u16> {
        self.resource_idx.borrow()
    }
}
impl ChromePak_Alias {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ChromePak_HeaderV5Part {
    pub(crate) _root: SharedType<ChromePak>,
    pub(crate) _parent: SharedType<ChromePak>,
    pub(crate) _self_shared: SharedType<Self>,
    encoding_padding: RefCell<Vec<u8>>,
    num_resources: RefCell<u16>,
    num_aliases: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ChromePak_HeaderV5Part {
    type Root = ChromePak;
    type Parent = ChromePak;

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
        *self_rc.encoding_padding.borrow_mut() = _io.read_bytes(3_usize)?;
        *self_rc.num_resources.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_aliases.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl ChromePak_HeaderV5Part {
}
impl ChromePak_HeaderV5Part {
    pub fn encoding_padding(&self) -> Ref<'_, Vec<u8>> {
        self.encoding_padding.borrow()
    }
}
impl ChromePak_HeaderV5Part {
    pub fn num_resources(&self) -> Ref<'_, u16> {
        self.num_resources.borrow()
    }
}
impl ChromePak_HeaderV5Part {
    pub fn num_aliases(&self) -> Ref<'_, u16> {
        self.num_aliases.borrow()
    }
}
impl ChromePak_HeaderV5Part {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ChromePak_Resource {
    pub(crate) _root: SharedType<ChromePak>,
    pub(crate) _parent: SharedType<ChromePak>,
    pub(crate) _self_shared: SharedType<Self>,
    idx: RefCell<i32>,
    has_body: RefCell<bool>,
    id: RefCell<u16>,
    ofs_body: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_body: Cell<bool>,
    body: RefCell<Vec<u8>>,
    f_len_body: Cell<bool>,
    len_body: RefCell<u32>,
}
impl KStruct for ChromePak_Resource {
    type Root = ChromePak;
    type Parent = ChromePak;

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
        *self_rc.id.borrow_mut() = _io.read_u2le()?;
        *self_rc.ofs_body.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl ChromePak_Resource {
    pub fn idx(&self) -> Ref<'_, i32> {
        self.idx.borrow()
    }
}
impl ChromePak_Resource {
    pub fn has_body(&self) -> Ref<'_, bool> {
        self.has_body.borrow()
    }
}
impl ChromePak_Resource {
    pub fn set_params(&mut self, idx: i32, has_body: bool) {
        *self.idx.borrow_mut() = idx;
        *self.has_body.borrow_mut() = has_body;
    }
}
impl ChromePak_Resource {

    /**
     * MUST NOT be accessed until the next `resource` is parsed
     */
    pub fn body(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        self.f_body.set(true);
        if *self.has_body() {
            let _pos = _io.pos();
            _io.seek(usize::try_from(*self.ofs_body())?)?;
            *self.body.borrow_mut() = _io.read_bytes(usize::try_from(*self.len_body()?)?)?;
            _io.seek(_pos)?;
        }
        Ok(self.body.borrow())
    }

    /**
     * MUST NOT be accessed until the next `resource` is parsed
     */
    pub fn len_body(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_len_body.get() {
            return Ok(self.len_body.borrow());
        }
        self.f_len_body.set(true);
        if *self.has_body() {
            *self.len_body.borrow_mut() = ((*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.resources().get(usize::try_from((*self.idx()).saturating_add(1_i32))?).ok_or(KError::CastError)?.ofs_body()).saturating_sub(*self.ofs_body())).try_into()?;
        }
        Ok(self.len_body.borrow())
    }
}
impl ChromePak_Resource {
    pub fn id(&self) -> Ref<'_, u16> {
        self.id.borrow()
    }
}
impl ChromePak_Resource {
    pub fn ofs_body(&self) -> Ref<'_, u32> {
        self.ofs_body.borrow()
    }
}
impl ChromePak_Resource {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
