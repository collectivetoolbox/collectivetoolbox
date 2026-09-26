// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * BSON, short for Binary JSON, is a binary-encoded serialization of JSON-like documents. Like JSON, BSON supports the embedding of documents and arrays within other documents and arrays. BSON also contains extensions that allow representation of data types that are not part of the JSON spec. For example, BSON has a Date type and a BinData type. BSON can be compared to binary interchange formats, like Protocol Buffers. BSON is more "schemaless" than Protocol Buffers, which can give it an advantage in flexibility but also a slight disadvantage in space efficiency (BSON has overhead for field names within the serialized data). BSON was designed to have the following three characteristics:
 *   * Lightweight. Keeping spatial overhead to a minimum is important for any data representation format, especially when used over the network.
 *   * Traversable. BSON is designed to be traversed easily. This is a vital property in its role as the primary data representation for MongoDB.
 *   * Efficient. Encoding data to BSON and decoding from BSON can be performed very quickly in most languages due to the use of C data types.
 */

#[derive(Default, Debug, Clone)]
pub struct Bson {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<i32>,
    fields: RefCell<OptRc<Bson_ElementsList>>,
    terminator: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    fields_raw: RefCell<Vec<u8>>,
}
impl KStruct for Bson {
    type Root = Bson;
    type Parent = Bson;

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
        *self_rc.len.borrow_mut() = _io.read_s4le()?;
        let _raw_fields = _io.read_bytes(usize::try_from((*self_rc.len()).saturating_sub(5_i32))?)?;
        *self_rc.fields_raw.borrow_mut() = _raw_fields.clone();
        let _io_fields = BytesReader::from(_raw_fields);
        let t = Self::read_into::<BytesReader, Bson_ElementsList>(&_io_fields, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.fields.borrow_mut() = t;
        *self_rc.terminator.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.terminator() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/2".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson {
}

/**
 * Total number of bytes comprising the document.
 */
impl Bson {
    pub fn len(&self) -> Ref<'_, i32> {
        self.len.borrow()
    }
}
impl Bson {
    pub fn fields(&self) -> Ref<'_, OptRc<Bson_ElementsList>> {
        self.fields.borrow()
    }
}
impl Bson {
    pub fn terminator(&self) -> Ref<'_, Vec<u8>> {
        self.terminator.borrow()
    }
}
impl Bson {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Bson {
    pub fn fields_raw(&self) -> Ref<'_, Vec<u8>> {
        self.fields_raw.borrow()
    }
}

/**
 * The BSON "binary" or "BinData" datatype is used to represent arrays of bytes. It is somewhat analogous to the Java notion of a ByteArray. BSON binary values have a subtype. This is used to indicate what kind of data is in the byte array. Subtypes from zero to 127 are predefined or reserved. Subtypes from 128-255 are user-defined.
 */

#[derive(Default, Debug, Clone)]
pub struct Bson_BinData {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_Element>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<i32>,
    subtype: RefCell<Bson_BinData_Subtype>,
    content: RefCell<Option<Bson_BinData_Content>>,
    _io: RefCell<BytesReader>,
    content_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Bson_BinData_Content {
    Bson_BinData_ByteArrayDeprecated(OptRc<Bson_BinData_ByteArrayDeprecated>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Bson_BinData_Content> for OptRc<Bson_BinData_ByteArrayDeprecated> {
    type Error = KError;
    fn try_from(v: &Bson_BinData_Content) -> Result<Self, Self::Error> {
        if let Bson_BinData_Content::Bson_BinData_ByteArrayDeprecated(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_BinData_ByteArrayDeprecated>> for Bson_BinData_Content {
    fn from(v: OptRc<Bson_BinData_ByteArrayDeprecated>) -> Self {
        Self::Bson_BinData_ByteArrayDeprecated(v)
    }
}
impl TryFrom<&Bson_BinData_Content> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Bson_BinData_Content) -> Result<Self, Self::Error> {
        if let Bson_BinData_Content::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Bson_BinData_Content {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Bson_BinData {
    type Root = Bson;
    type Parent = Bson_Element;

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
        *self_rc.len.borrow_mut() = _io.read_s4le()?;
        *self_rc.subtype.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        match *self_rc.subtype() {
            Bson_BinData_Subtype::ByteArrayDeprecated => {
                *self_rc.content_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len())?)?.into();
                let content_raw = self_rc.content_raw.borrow();
                let _t_content_raw_io = BytesReader::from(content_raw.clone());
                let t = Self::read_into::<BytesReader, Bson_BinData_ByteArrayDeprecated>(&_t_content_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.content.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_BinData {
}
impl Bson_BinData {
    pub fn len(&self) -> Ref<'_, i32> {
        self.len.borrow()
    }
}
impl Bson_BinData {
    pub fn subtype(&self) -> Ref<'_, Bson_BinData_Subtype> {
        self.subtype.borrow()
    }
}
impl Bson_BinData {
    pub fn content(&self) -> Ref<'_, Option<Bson_BinData_Content>> {
        self.content.borrow()
    }
}
impl Bson_BinData {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Bson_BinData {
    pub fn content_raw(&self) -> Ref<'_, Vec<u8>> {
        self.content_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bson_BinData_Subtype {
    Generic,
    Function,
    ByteArrayDeprecated,
    UuidDeprecated,
    Uuid,
    Md5,
    Custom,
    Unknown(i64),
}

impl TryFrom<i64> for Bson_BinData_Subtype {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Bson_BinData_Subtype> {
        match flag {
            0 => Ok(Bson_BinData_Subtype::Generic),
            1 => Ok(Bson_BinData_Subtype::Function),
            2 => Ok(Bson_BinData_Subtype::ByteArrayDeprecated),
            3 => Ok(Bson_BinData_Subtype::UuidDeprecated),
            4 => Ok(Bson_BinData_Subtype::Uuid),
            5 => Ok(Bson_BinData_Subtype::Md5),
            128 => Ok(Bson_BinData_Subtype::Custom),
            _ => Ok(Bson_BinData_Subtype::Unknown(flag)),
        }
    }
}

impl From<&Bson_BinData_Subtype> for i64 {
    fn from(v: &Bson_BinData_Subtype) -> Self {
        match *v {
            Bson_BinData_Subtype::Generic => 0,
            Bson_BinData_Subtype::Function => 1,
            Bson_BinData_Subtype::ByteArrayDeprecated => 2,
            Bson_BinData_Subtype::UuidDeprecated => 3,
            Bson_BinData_Subtype::Uuid => 4,
            Bson_BinData_Subtype::Md5 => 5,
            Bson_BinData_Subtype::Custom => 128,
            Bson_BinData_Subtype::Unknown(v) => v
        }
    }
}

impl Default for Bson_BinData_Subtype {
    fn default() -> Self { Bson_BinData_Subtype::Unknown(0) }
}


/**
 * The BSON "binary" or "BinData" datatype is used to represent arrays of bytes. It is somewhat analogous to the Java notion of a ByteArray. BSON binary values have a subtype. This is used to indicate what kind of data is in the byte array. Subtypes from zero to 127 are predefined or reserved. Subtypes from 128-255 are user-defined.
 */

#[derive(Default, Debug, Clone)]
pub struct Bson_BinData_ByteArrayDeprecated {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_BinData>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<i32>,
    content: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    content_raw: RefCell<Vec<u8>>,
}
impl KStruct for Bson_BinData_ByteArrayDeprecated {
    type Root = Bson;
    type Parent = Bson_BinData;

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
        *self_rc.len.borrow_mut() = _io.read_s4le()?;
        *self_rc.content.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len())?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_BinData_ByteArrayDeprecated {
}
impl Bson_BinData_ByteArrayDeprecated {
    pub fn len(&self) -> Ref<'_, i32> {
        self.len.borrow()
    }
}
impl Bson_BinData_ByteArrayDeprecated {
    pub fn content(&self) -> Ref<'_, Vec<u8>> {
        self.content.borrow()
    }
}
impl Bson_BinData_ByteArrayDeprecated {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Bson_BinData_ByteArrayDeprecated {
    pub fn content_raw(&self) -> Ref<'_, Vec<u8>> {
        self.content_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bson_CodeWithScope {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_Element>,
    pub(crate) _self_shared: SharedType<Self>,
    id: RefCell<i32>,
    source: RefCell<OptRc<Bson_String>>,
    scope: RefCell<OptRc<Bson>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_CodeWithScope {
    type Root = Bson;
    type Parent = Bson_Element;

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
        *self_rc.id.borrow_mut() = _io.read_s4le()?;
        let t = Self::read_into::<_, Bson_String>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.source.borrow_mut() = t;
        let t = Self::read_into::<_, Bson>(&*_io, Some(self_rc._root.clone()), Some(self_rc._root.clone()))?.into();
        *self_rc.scope.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_CodeWithScope {
}
impl Bson_CodeWithScope {
    pub fn id(&self) -> Ref<'_, i32> {
        self.id.borrow()
    }
}
impl Bson_CodeWithScope {
    pub fn source(&self) -> Ref<'_, OptRc<Bson_String>> {
        self.source.borrow()
    }
}

/**
 * mapping from identifiers to values, representing the scope in which the string should be evaluated.
 */
impl Bson_CodeWithScope {
    pub fn scope(&self) -> Ref<'_, OptRc<Bson>> {
        self.scope.borrow()
    }
}
impl Bson_CodeWithScope {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bson_Cstring {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_Cstring {
    type Root = Bson;
    type Parent = KStructUnit;

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
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_Cstring {
}

/**
 * MUST NOT contain '\x00', hence it is not full UTF-8.
 */
impl Bson_Cstring {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl Bson_Cstring {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bson_DbPointer {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_Element>,
    pub(crate) _self_shared: SharedType<Self>,
    namespace: RefCell<OptRc<Bson_String>>,
    id: RefCell<OptRc<Bson_ObjectId>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_DbPointer {
    type Root = Bson;
    type Parent = Bson_Element;

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
        let t = Self::read_into::<_, Bson_String>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.namespace.borrow_mut() = t;
        let t = Self::read_into::<_, Bson_ObjectId>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.id.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_DbPointer {
}
impl Bson_DbPointer {
    pub fn namespace(&self) -> Ref<'_, OptRc<Bson_String>> {
        self.namespace.borrow()
    }
}
impl Bson_DbPointer {
    pub fn id(&self) -> Ref<'_, OptRc<Bson_ObjectId>> {
        self.id.borrow()
    }
}
impl Bson_DbPointer {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bson_Element {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_ElementsList>,
    pub(crate) _self_shared: SharedType<Self>,
    type_byte: RefCell<Bson_Element_BsonType>,
    name: RefCell<OptRc<Bson_Cstring>>,
    content: RefCell<Option<Bson_Element_Content>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum Bson_Element_Content {
    Bson(OptRc<Bson>),
    Bson_BinData(OptRc<Bson_BinData>),
    U1(u8),
    Bson_CodeWithScope(OptRc<Bson_CodeWithScope>),
    Bson_DbPointer(OptRc<Bson_DbPointer>),
    Bson_String(OptRc<Bson_String>),
    Bson_F16(OptRc<Bson_F16>),
    F8(f64),
    S4(i32),
    S8(i64),
    Bson_ObjectId(OptRc<Bson_ObjectId>),
    Bson_RegEx(OptRc<Bson_RegEx>),
    Bson_Timestamp(OptRc<Bson_Timestamp>),
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson>> for Bson_Element_Content {
    fn from(v: OptRc<Bson>) -> Self {
        Self::Bson(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_BinData> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_BinData(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_BinData>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_BinData>) -> Self {
        Self::Bson_BinData(v)
    }
}
impl TryFrom<&Bson_Element_Content> for u8 {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::U1(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<u8> for Bson_Element_Content {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_CodeWithScope> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_CodeWithScope(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_CodeWithScope>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_CodeWithScope>) -> Self {
        Self::Bson_CodeWithScope(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_DbPointer> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_DbPointer(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_DbPointer>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_DbPointer>) -> Self {
        Self::Bson_DbPointer(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_String> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_String(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_String>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_String>) -> Self {
        Self::Bson_String(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_F16> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_F16(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_F16>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_F16>) -> Self {
        Self::Bson_F16(v)
    }
}
impl TryFrom<&Bson_Element_Content> for f64 {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::F8(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<f64> for Bson_Element_Content {
    fn from(v: f64) -> Self {
        Self::F8(v)
    }
}
impl TryFrom<&Bson_Element_Content> for i32 {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::S4(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<i32> for Bson_Element_Content {
    fn from(v: i32) -> Self {
        Self::S4(v)
    }
}
impl TryFrom<&Bson_Element_Content> for i64 {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::S8(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<i64> for Bson_Element_Content {
    fn from(v: i64) -> Self {
        Self::S8(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_ObjectId> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_ObjectId(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_ObjectId>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_ObjectId>) -> Self {
        Self::Bson_ObjectId(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_RegEx> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_RegEx(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_RegEx>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_RegEx>) -> Self {
        Self::Bson_RegEx(v)
    }
}
impl TryFrom<&Bson_Element_Content> for OptRc<Bson_Timestamp> {
    type Error = KError;
    fn try_from(v: &Bson_Element_Content) -> Result<Self, Self::Error> {
        if let Bson_Element_Content::Bson_Timestamp(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Bson_Timestamp>> for Bson_Element_Content {
    fn from(v: OptRc<Bson_Timestamp>) -> Self {
        Self::Bson_Timestamp(v)
    }
}
impl KStruct for Bson_Element {
    type Root = Bson;
    type Parent = Bson_ElementsList;

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
        *self_rc.type_byte.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        let t = Self::read_into::<_, Bson_Cstring>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.name.borrow_mut() = t;
        match *self_rc.type_byte() {
            Bson_Element_BsonType::Array => {
                let t = Self::read_into::<_, Bson>(&*_io, Some(self_rc._root.clone()), Some(self_rc._root.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::BinData => {
                let t = Self::read_into::<_, Bson_BinData>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::Boolean => {
                *self_rc.content.borrow_mut() = Some(_io.read_u1()?.into());
            }
            Bson_Element_BsonType::CodeWithScope => {
                let t = Self::read_into::<_, Bson_CodeWithScope>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::DbPointer => {
                let t = Self::read_into::<_, Bson_DbPointer>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::Document => {
                let t = Self::read_into::<_, Bson>(&*_io, Some(self_rc._root.clone()), Some(self_rc._root.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::Javascript => {
                let t = Self::read_into::<_, Bson_String>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::NumberDecimal => {
                let t = Self::read_into::<_, Bson_F16>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::NumberDouble => {
                *self_rc.content.borrow_mut() = Some(_io.read_f8le()?.into());
            }
            Bson_Element_BsonType::NumberInt => {
                *self_rc.content.borrow_mut() = Some(_io.read_s4le()?.into());
            }
            Bson_Element_BsonType::NumberLong => {
                *self_rc.content.borrow_mut() = Some(_io.read_s8le()?.into());
            }
            Bson_Element_BsonType::ObjectId => {
                let t = Self::read_into::<_, Bson_ObjectId>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::RegEx => {
                let t = Self::read_into::<_, Bson_RegEx>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::String => {
                let t = Self::read_into::<_, Bson_String>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::Symbol => {
                let t = Self::read_into::<_, Bson_String>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::Timestamp => {
                let t = Self::read_into::<_, Bson_Timestamp>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Bson_Element_BsonType::UtcDatetime => {
                *self_rc.content.borrow_mut() = Some(_io.read_s8le()?.into());
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_Element {
}
impl Bson_Element {
    pub fn type_byte(&self) -> Ref<'_, Bson_Element_BsonType> {
        self.type_byte.borrow()
    }
}
impl Bson_Element {
    pub fn name(&self) -> Ref<'_, OptRc<Bson_Cstring>> {
        self.name.borrow()
    }
}
impl Bson_Element {
    pub fn content(&self) -> Ref<'_, Option<Bson_Element_Content>> {
        self.content.borrow()
    }
}
impl Bson_Element {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bson_Element_BsonType {
    MinKey,
    EndOfObject,
    NumberDouble,
    String,
    Document,
    Array,
    BinData,
    Undefined,
    ObjectId,
    Boolean,
    UtcDatetime,
    JstNull,
    RegEx,
    DbPointer,
    Javascript,
    Symbol,
    CodeWithScope,
    NumberInt,
    Timestamp,
    NumberLong,
    NumberDecimal,
    MaxKey,
    Unknown(i64),
}

impl TryFrom<i64> for Bson_Element_BsonType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Bson_Element_BsonType> {
        match flag {
            -1 => Ok(Bson_Element_BsonType::MinKey),
            0 => Ok(Bson_Element_BsonType::EndOfObject),
            1 => Ok(Bson_Element_BsonType::NumberDouble),
            2 => Ok(Bson_Element_BsonType::String),
            3 => Ok(Bson_Element_BsonType::Document),
            4 => Ok(Bson_Element_BsonType::Array),
            5 => Ok(Bson_Element_BsonType::BinData),
            6 => Ok(Bson_Element_BsonType::Undefined),
            7 => Ok(Bson_Element_BsonType::ObjectId),
            8 => Ok(Bson_Element_BsonType::Boolean),
            9 => Ok(Bson_Element_BsonType::UtcDatetime),
            10 => Ok(Bson_Element_BsonType::JstNull),
            11 => Ok(Bson_Element_BsonType::RegEx),
            12 => Ok(Bson_Element_BsonType::DbPointer),
            13 => Ok(Bson_Element_BsonType::Javascript),
            14 => Ok(Bson_Element_BsonType::Symbol),
            15 => Ok(Bson_Element_BsonType::CodeWithScope),
            16 => Ok(Bson_Element_BsonType::NumberInt),
            17 => Ok(Bson_Element_BsonType::Timestamp),
            18 => Ok(Bson_Element_BsonType::NumberLong),
            19 => Ok(Bson_Element_BsonType::NumberDecimal),
            127 => Ok(Bson_Element_BsonType::MaxKey),
            _ => Ok(Bson_Element_BsonType::Unknown(flag)),
        }
    }
}

impl From<&Bson_Element_BsonType> for i64 {
    fn from(v: &Bson_Element_BsonType) -> Self {
        match *v {
            Bson_Element_BsonType::MinKey => -1,
            Bson_Element_BsonType::EndOfObject => 0,
            Bson_Element_BsonType::NumberDouble => 1,
            Bson_Element_BsonType::String => 2,
            Bson_Element_BsonType::Document => 3,
            Bson_Element_BsonType::Array => 4,
            Bson_Element_BsonType::BinData => 5,
            Bson_Element_BsonType::Undefined => 6,
            Bson_Element_BsonType::ObjectId => 7,
            Bson_Element_BsonType::Boolean => 8,
            Bson_Element_BsonType::UtcDatetime => 9,
            Bson_Element_BsonType::JstNull => 10,
            Bson_Element_BsonType::RegEx => 11,
            Bson_Element_BsonType::DbPointer => 12,
            Bson_Element_BsonType::Javascript => 13,
            Bson_Element_BsonType::Symbol => 14,
            Bson_Element_BsonType::CodeWithScope => 15,
            Bson_Element_BsonType::NumberInt => 16,
            Bson_Element_BsonType::Timestamp => 17,
            Bson_Element_BsonType::NumberLong => 18,
            Bson_Element_BsonType::NumberDecimal => 19,
            Bson_Element_BsonType::MaxKey => 127,
            Bson_Element_BsonType::Unknown(v) => v
        }
    }
}

impl Default for Bson_Element_BsonType {
    fn default() -> Self { Bson_Element_BsonType::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Bson_ElementsList {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson>,
    pub(crate) _self_shared: SharedType<Self>,
    elements: RefCell<Vec<OptRc<Bson_Element>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_ElementsList {
    type Root = Bson;
    type Parent = Bson;

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
        *self_rc.elements.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Bson_Element>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.elements.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_ElementsList {
}
impl Bson_ElementsList {
    pub fn elements(&self) -> Ref<'_, Vec<OptRc<Bson_Element>>> {
        self.elements.borrow()
    }
}
impl Bson_ElementsList {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * 128-bit IEEE 754-2008 decimal floating point
 */

#[derive(Default, Debug, Clone)]
pub struct Bson_F16 {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_Element>,
    pub(crate) _self_shared: SharedType<Self>,
    str: RefCell<bool>,
    exponent: RefCell<u64>,
    significand_hi: RefCell<u64>,
    significand_lo: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_F16 {
    type Root = Bson;
    type Parent = Bson_Element;

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
        *self_rc.str.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.exponent.borrow_mut() = _io.read_bits_int_be(15)?;
        *self_rc.significand_hi.borrow_mut() = _io.read_bits_int_be(49)?;
        io.align_to_byte()?;
        *self_rc.significand_lo.borrow_mut() = _io.read_u8le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_F16 {
}
impl Bson_F16 {
    pub fn str(&self) -> Ref<'_, bool> {
        self.str.borrow()
    }
}
impl Bson_F16 {
    pub fn exponent(&self) -> Ref<'_, u64> {
        self.exponent.borrow()
    }
}
impl Bson_F16 {
    pub fn significand_hi(&self) -> Ref<'_, u64> {
        self.significand_hi.borrow()
    }
}
impl Bson_F16 {
    pub fn significand_lo(&self) -> Ref<'_, u64> {
        self.significand_lo.borrow()
    }
}
impl Bson_F16 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://www.mongodb.com/docs/manual/reference/method/ObjectId/> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Bson_ObjectId {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    epoch_time: RefCell<u32>,
    machine_id: RefCell<OptRc<Bson_U3>>,
    process_id: RefCell<u16>,
    counter: RefCell<OptRc<Bson_U3>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_ObjectId {
    type Root = Bson;
    type Parent = KStructUnit;

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
        *self_rc.epoch_time.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, Bson_U3>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.machine_id.borrow_mut() = t;
        *self_rc.process_id.borrow_mut() = _io.read_u2le()?;
        let t = Self::read_into::<_, Bson_U3>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.counter.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_ObjectId {
}

/**
 * seconds since the Unix epoch
 */
impl Bson_ObjectId {
    pub fn epoch_time(&self) -> Ref<'_, u32> {
        self.epoch_time.borrow()
    }
}
impl Bson_ObjectId {
    pub fn machine_id(&self) -> Ref<'_, OptRc<Bson_U3>> {
        self.machine_id.borrow()
    }
}
impl Bson_ObjectId {
    pub fn process_id(&self) -> Ref<'_, u16> {
        self.process_id.borrow()
    }
}

/**
 * counter, starting with a random value.
 */
impl Bson_ObjectId {
    pub fn counter(&self) -> Ref<'_, OptRc<Bson_U3>> {
        self.counter.borrow()
    }
}
impl Bson_ObjectId {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bson_RegEx {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_Element>,
    pub(crate) _self_shared: SharedType<Self>,
    pattern: RefCell<OptRc<Bson_Cstring>>,
    options: RefCell<OptRc<Bson_Cstring>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_RegEx {
    type Root = Bson;
    type Parent = Bson_Element;

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
        let t = Self::read_into::<_, Bson_Cstring>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.pattern.borrow_mut() = t;
        let t = Self::read_into::<_, Bson_Cstring>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.options.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_RegEx {
}
impl Bson_RegEx {
    pub fn pattern(&self) -> Ref<'_, OptRc<Bson_Cstring>> {
        self.pattern.borrow()
    }
}
impl Bson_RegEx {
    pub fn options(&self) -> Ref<'_, OptRc<Bson_Cstring>> {
        self.options.borrow()
    }
}
impl Bson_RegEx {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bson_String {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<i32>,
    str: RefCell<String>,
    terminator: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    str_raw: RefCell<Vec<u8>>,
}
impl KStruct for Bson_String {
    type Root = Bson;
    type Parent = KStructUnit;

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
        *self_rc.len.borrow_mut() = _io.read_s4le()?;
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from((*self_rc.len()).saturating_sub(1_i32))?)?, "UTF-8")?;
        *self_rc.terminator.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.terminator() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/string/seq/2".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_String {
}
impl Bson_String {
    pub fn len(&self) -> Ref<'_, i32> {
        self.len.borrow()
    }
}
impl Bson_String {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl Bson_String {
    pub fn terminator(&self) -> Ref<'_, Vec<u8>> {
        self.terminator.borrow()
    }
}
impl Bson_String {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Bson_String {
    pub fn str_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_raw.borrow()
    }
}

/**
 * Special internal type used by MongoDB replication and sharding. First 4 bytes are an increment, second 4 are a timestamp.
 */

#[derive(Default, Debug, Clone)]
pub struct Bson_Timestamp {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_Element>,
    pub(crate) _self_shared: SharedType<Self>,
    increment: RefCell<u32>,
    timestamp: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Bson_Timestamp {
    type Root = Bson;
    type Parent = Bson_Element;

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
        *self_rc.increment.borrow_mut() = _io.read_u4le()?;
        *self_rc.timestamp.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_Timestamp {
}
impl Bson_Timestamp {
    pub fn increment(&self) -> Ref<'_, u32> {
        self.increment.borrow()
    }
}
impl Bson_Timestamp {
    pub fn timestamp(&self) -> Ref<'_, u32> {
        self.timestamp.borrow()
    }
}
impl Bson_Timestamp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Implements unsigned 24-bit (3 byte) integer.
 */

#[derive(Default, Debug, Clone)]
pub struct Bson_U3 {
    pub(crate) _root: SharedType<Bson>,
    pub(crate) _parent: SharedType<Bson_ObjectId>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    b2: RefCell<u8>,
    b3: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<i32>,
}
impl KStruct for Bson_U3 {
    type Root = Bson;
    type Parent = Bson_ObjectId;

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
        *self_rc.b1.borrow_mut() = _io.read_u1()?;
        *self_rc.b2.borrow_mut() = _io.read_u1()?;
        *self_rc.b3.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Bson_U3 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = (((((i32::from(*self.b1())) | ((i32::from(*self.b2())).wrapping_shl(8_u32)))) | ((i32::from(*self.b3())).wrapping_shl(16_u32)))).try_into()?;
        Ok(self.value.borrow())
    }
}
impl Bson_U3 {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl Bson_U3 {
    pub fn b2(&self) -> Ref<'_, u8> {
        self.b2.borrow()
    }
}
impl Bson_U3 {
    pub fn b3(&self) -> Ref<'_, u8> {
        self.b3.borrow()
    }
}
impl Bson_U3 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
