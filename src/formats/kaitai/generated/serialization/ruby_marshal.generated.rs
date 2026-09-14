// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Ruby's Marshal module allows serialization and deserialization of
 * many standard and arbitrary Ruby objects in a compact binary
 * format. It is relatively fast, available in stdlibs standard and
 * allows conservation of language-specific properties (such as symbols
 * or encoding-aware strings).
 *
 * Feature-wise, it is comparable to other language-specific
 * implementations, such as:
 *
 * * Java's
 *   [Serializable](https://docs.oracle.com/javase/8/docs/api/java/io/Serializable.html)
 * * .NET
 *   [BinaryFormatter](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.serialization.formatters.binary.binaryformatter?view=net-7.0)
 * * Python's
 *   [marshal](https://docs.python.org/3/library/marshal.html),
 *   [pickle](https://docs.python.org/3/library/pickle.html) and
 *   [shelve](https://docs.python.org/3/library/shelve.html)
 *
 * From internal perspective, serialized stream consists of a simple
 * magic header and a record.
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Stream+Format> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<Vec<u8>>,
    records: RefCell<OptRc<RubyMarshal_Record>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RubyMarshal {
    type Root = RubyMarshal;
    type Parent = RubyMarshal;

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
        *self_rc.version.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.version() == vec![0x4u8, 0x8u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        let t = Self::read_into::<_, RubyMarshal_Record>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.records.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal {
}
impl RubyMarshal {
    pub fn version(&self) -> Ref<'_, Vec<u8>> {
        self.version.borrow()
    }
}
impl RubyMarshal {
    pub fn records(&self) -> Ref<'_, OptRc<RubyMarshal_Record>> {
        self.records.borrow()
    }
}
impl RubyMarshal {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RubyMarshal_Codes {
    RubyString,
    ConstNil,
    RubySymbol,
    RubySymbolLink,
    RubyObjectLink,
    ConstFalse,
    InstanceVar,
    RubyStruct,
    ConstTrue,
    RubyArray,
    PackedInt,
    Bignum,
    RubyHash,
    Unknown(i64),
}

impl TryFrom<i64> for RubyMarshal_Codes {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<RubyMarshal_Codes> {
        match flag {
            34 => Ok(RubyMarshal_Codes::RubyString),
            48 => Ok(RubyMarshal_Codes::ConstNil),
            58 => Ok(RubyMarshal_Codes::RubySymbol),
            59 => Ok(RubyMarshal_Codes::RubySymbolLink),
            64 => Ok(RubyMarshal_Codes::RubyObjectLink),
            70 => Ok(RubyMarshal_Codes::ConstFalse),
            73 => Ok(RubyMarshal_Codes::InstanceVar),
            83 => Ok(RubyMarshal_Codes::RubyStruct),
            84 => Ok(RubyMarshal_Codes::ConstTrue),
            91 => Ok(RubyMarshal_Codes::RubyArray),
            105 => Ok(RubyMarshal_Codes::PackedInt),
            108 => Ok(RubyMarshal_Codes::Bignum),
            123 => Ok(RubyMarshal_Codes::RubyHash),
            _ => Ok(RubyMarshal_Codes::Unknown(flag)),
        }
    }
}

impl From<&RubyMarshal_Codes> for i64 {
    fn from(v: &RubyMarshal_Codes) -> Self {
        match *v {
            RubyMarshal_Codes::RubyString => 34,
            RubyMarshal_Codes::ConstNil => 48,
            RubyMarshal_Codes::RubySymbol => 58,
            RubyMarshal_Codes::RubySymbolLink => 59,
            RubyMarshal_Codes::RubyObjectLink => 64,
            RubyMarshal_Codes::ConstFalse => 70,
            RubyMarshal_Codes::InstanceVar => 73,
            RubyMarshal_Codes::RubyStruct => 83,
            RubyMarshal_Codes::ConstTrue => 84,
            RubyMarshal_Codes::RubyArray => 91,
            RubyMarshal_Codes::PackedInt => 105,
            RubyMarshal_Codes::Bignum => 108,
            RubyMarshal_Codes::RubyHash => 123,
            RubyMarshal_Codes::Unknown(v) => v
        }
    }
}

impl Default for RubyMarshal_Codes {
    fn default() -> Self { RubyMarshal_Codes::Unknown(0) }
}


/**
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Bignum> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_Bignum {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    sign: RefCell<u8>,
    len_div_2: RefCell<OptRc<RubyMarshal_PackedInt>>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
impl KStruct for RubyMarshal_Bignum {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        *self_rc.sign.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.len_div_2.borrow_mut() = t;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc.len_div_2().value()?).saturating_mul(2_i32))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_Bignum {
}

/**
 * A single byte containing `+` for a positive value or `-` for a negative value.
 */
impl RubyMarshal_Bignum {
    pub fn sign(&self) -> Ref<'_, u8> {
        self.sign.borrow()
    }
}

/**
 * Length of bignum body, divided by 2.
 */
impl RubyMarshal_Bignum {
    pub fn len_div_2(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.len_div_2.borrow()
    }
}

/**
 * Bytes that represent the number, see ruby-lang.org docs for reconstruction algorithm.
 */
impl RubyMarshal_Bignum {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl RubyMarshal_Bignum {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl RubyMarshal_Bignum {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

/**
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Instance+Variables> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_InstanceVar {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    obj: RefCell<OptRc<RubyMarshal_Record>>,
    num_vars: RefCell<OptRc<RubyMarshal_PackedInt>>,
    vars: RefCell<Vec<OptRc<RubyMarshal_Pair>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RubyMarshal_InstanceVar {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        let t = Self::read_into::<_, RubyMarshal_Record>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.obj.borrow_mut() = t;
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.num_vars.borrow_mut() = t;
        *self_rc.vars.borrow_mut() = Vec::new();
        let l_vars = usize::try_from(*self_rc.num_vars().value()?)?;
        for _i in 0_usize..l_vars {
            let t = Self::read_into::<_, RubyMarshal_Pair>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.vars.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_InstanceVar {
}
impl RubyMarshal_InstanceVar {
    pub fn obj(&self) -> Ref<'_, OptRc<RubyMarshal_Record>> {
        self.obj.borrow()
    }
}
impl RubyMarshal_InstanceVar {
    pub fn num_vars(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.num_vars.borrow()
    }
}
impl RubyMarshal_InstanceVar {
    pub fn vars(&self) -> Ref<'_, Vec<OptRc<RubyMarshal_Pair>>> {
        self.vars.borrow()
    }
}
impl RubyMarshal_InstanceVar {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Ruby uses sophisticated system to pack integers: first `code`
 * byte either determines packing scheme or carries encoded
 * immediate value (thus allowing smaller values from -123 to 122
 * (inclusive) to take only one byte. There are 11 encoding schemes
 * in total:
 *
 * * 0 is encoded specially (as 0)
 * * 1..122 are encoded as immediate value with a shift
 * * 123..255 are encoded with code of 0x01 and 1 extra byte
 * * 0x100..0xffff are encoded with code of 0x02 and 2 extra bytes
 * * 0x10000..0xffffff are encoded with code of 0x03 and 3 extra
 *   bytes
 * * 0x1000000..0xffffffff are encoded with code of 0x04 and 4
 *   extra bytes
 * * -123..-1 are encoded as immediate value with another shift
 * * -256..-124 are encoded with code of 0xff and 1 extra byte
 * * -0x10000..-257 are encoded with code of 0xfe and 2 extra bytes
 * * -0x1000000..0x10001 are encoded with code of 0xfd and 3 extra
 *    bytes
 * * -0x40000000..-0x1000001 are encoded with code of 0xfc and 4
 *    extra bytes
 *
 * Values beyond that are serialized as bignum (even if they
 * technically might be not Bignum class in Ruby implementation,
 * i.e. if they fit into 64 bits on a 64-bit platform).
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Fixnum+and+long> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_PackedInt {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    encoded: RefCell<Option<RubyMarshal_PackedInt_Encoded>>,
    encoded2: RefCell<Option<RubyMarshal_PackedInt_Encoded2>>,
    _io: RefCell<BytesReader>,
    f_is_immediate: Cell<bool>,
    is_immediate: RefCell<bool>,
    f_value: Cell<bool>,
    value: RefCell<i32>,
}
#[derive(Debug, Clone)]
pub enum RubyMarshal_PackedInt_Encoded {
    U1(u8),
    U2(u16),
    U4(u32),
}
impl From<u8> for RubyMarshal_PackedInt_Encoded {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<u16> for RubyMarshal_PackedInt_Encoded {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl From<u32> for RubyMarshal_PackedInt_Encoded {
    fn from(v: u32) -> Self {
        Self::U4(v)
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded::U1(v) => Ok(i64::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U2(v) => Ok(i64::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U4(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded> for u16 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded::U1(v) => Ok(u16::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U2(v) => Ok(u16::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U4(v) => Ok(u16::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded> for u32 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded::U1(v) => Ok(u32::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U2(v) => Ok(u32::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U4(v) => Ok(u32::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded::U1(v) => Ok(u64::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U2(v) => Ok(u64::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U4(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded> for u8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded::U1(v) => Ok(u8::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U2(v) => Ok(u8::try_from(*v)?),
            RubyMarshal_PackedInt_Encoded::U4(v) => Ok(u8::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded> for usize {
    type Error = KError;
    fn try_from(e: &RubyMarshal_PackedInt_Encoded) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded::U1(v) => Ok(usize::from(*v)),
            RubyMarshal_PackedInt_Encoded::U2(v) => Ok(usize::from(*v)),
            RubyMarshal_PackedInt_Encoded::U4(v) => Ok(usize::try_from(*v)?),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RubyMarshal_PackedInt_Encoded2 {
    U1(u8),
}
impl From<u8> for RubyMarshal_PackedInt_Encoded2 {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<&RubyMarshal_PackedInt_Encoded2> for u8 {
    fn from(e: &RubyMarshal_PackedInt_Encoded2) -> Self {
        let RubyMarshal_PackedInt_Encoded2::U1(v) = e;
        *v
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded2> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded2) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded2::U1(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded2> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded2) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded2::U1(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded2> for u8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &RubyMarshal_PackedInt_Encoded2) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded2::U1(v) => Ok(u8::try_from(*v)?),
        }
    }
}
impl TryFrom<&RubyMarshal_PackedInt_Encoded2> for usize {
    type Error = KError;
    fn try_from(e: &RubyMarshal_PackedInt_Encoded2) -> Result<Self, Self::Error> {
        match e {
            RubyMarshal_PackedInt_Encoded2::U1(v) => Ok(usize::from(*v)),
        }
    }
}

impl KStruct for RubyMarshal_PackedInt {
    type Root = RubyMarshal;
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
        *self_rc.code.borrow_mut() = _io.read_u1()?;
        match *self_rc.code() {
            1 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u1()?.into());
            }
            2 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            252 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u4le()?.into());
            }
            253 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            254 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            255 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u1()?.into());
            }
            3 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            4 => {
                *self_rc.encoded.borrow_mut() = Some(_io.read_u4le()?.into());
            }
            _ => {}
        }
        match *self_rc.code() {
            253 => {
                *self_rc.encoded2.borrow_mut() = Some(_io.read_u1()?.into());
            }
            3 => {
                *self_rc.encoded2.borrow_mut() = Some(_io.read_u1()?.into());
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_PackedInt {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_immediate(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_immediate.get() {
            return Ok(self.is_immediate.borrow());
        }
        self.f_is_immediate.set(true);
        *self.is_immediate.borrow_mut() = ( ((((to_i128(*self.code())) > (to_i128(4)))) && (((to_i128(*self.code())) < (to_i128(252))))) ).try_into()?;
        Ok(self.is_immediate.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = (if *self.is_immediate()? { u32::try_from(if ((to_i128(*self.code())) < (to_i128(128))) { (i32::from(*self.code())).saturating_sub(5_i32) } else { (4_i32).saturating_sub(((i32::from(!(*self.code()))) & (127_i32))) })? } else { if ((to_i128(*self.code())) == (to_i128(0))) { 0_u32 } else { if ((to_i128(*self.code())) == (to_i128(255))) { (self.encoded()).saturating_sub(256_u32) } else { if ((to_i128(*self.code())) == (to_i128(254))) { (self.encoded()).saturating_sub(65536_u32) } else { if ((to_i128(*self.code())) == (to_i128(253))) { (((u32::try_from((i32::from(self.encoded2())).wrapping_shl(16_u32))?) | (self.encoded()))).saturating_sub(16777216_u32) } else { if ((to_i128(*self.code())) == (to_i128(3))) { ((u32::try_from((i32::from(self.encoded2())).wrapping_shl(16_u32))?) | (self.encoded())) } else { self.encoded() } } } } } }).try_into()?;
        Ok(self.value.borrow())
    }
}
impl RubyMarshal_PackedInt {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl RubyMarshal_PackedInt {
    pub fn encoded(&self) -> u32 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.encoded.borrow().as_ref().and_then(|v| u32::try_from(v).ok()).unwrap_or(0)
    }
    pub fn encoded_enum(&self) -> Ref<'_, Option<RubyMarshal_PackedInt_Encoded>> {
        self.encoded.borrow()
    }
}

/**
 * One extra byte for 3-byte integers (0x03 and 0xfd), as
 * there is no standard `u3` type in KS.
 */
impl RubyMarshal_PackedInt {
    pub fn encoded2(&self) -> u8 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.encoded2.borrow().as_ref().and_then(|v| u8::try_from(v).ok()).unwrap_or(0)
    }
    pub fn encoded2_enum(&self) -> Ref<'_, Option<RubyMarshal_PackedInt_Encoded2>> {
        self.encoded2.borrow()
    }
}
impl RubyMarshal_PackedInt {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_Pair {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    key: RefCell<OptRc<RubyMarshal_Record>>,
    value: RefCell<OptRc<RubyMarshal_Record>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RubyMarshal_Pair {
    type Root = RubyMarshal;
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
        let t = Self::read_into::<_, RubyMarshal_Record>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.key.borrow_mut() = t;
        let t = Self::read_into::<_, RubyMarshal_Record>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.value.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_Pair {
}
impl RubyMarshal_Pair {
    pub fn key(&self) -> Ref<'_, OptRc<RubyMarshal_Record>> {
        self.key.borrow()
    }
}
impl RubyMarshal_Pair {
    pub fn value(&self) -> Ref<'_, OptRc<RubyMarshal_Record>> {
        self.value.borrow()
    }
}
impl RubyMarshal_Pair {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Each record starts with a single byte that determines its type
 * (`code`) and contents. If necessary, additional info as parsed
 * as `body`, to be determined by `code`.
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_Record {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<RubyMarshal_Codes>,
    body: RefCell<Option<RubyMarshal_Record_Body>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum RubyMarshal_Record_Body {
    RubyMarshal_Bignum(OptRc<RubyMarshal_Bignum>),
    RubyMarshal_InstanceVar(OptRc<RubyMarshal_InstanceVar>),
    RubyMarshal_PackedInt(OptRc<RubyMarshal_PackedInt>),
    RubyMarshal_RubyArray(OptRc<RubyMarshal_RubyArray>),
    RubyMarshal_RubyHash(OptRc<RubyMarshal_RubyHash>),
    RubyMarshal_RubyString(OptRc<RubyMarshal_RubyString>),
    RubyMarshal_RubyStruct(OptRc<RubyMarshal_RubyStruct>),
    RubyMarshal_RubySymbol(OptRc<RubyMarshal_RubySymbol>),
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_Bignum> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_Bignum(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_Bignum>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_Bignum>) -> Self {
        Self::RubyMarshal_Bignum(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_InstanceVar> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_InstanceVar(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_InstanceVar>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_InstanceVar>) -> Self {
        Self::RubyMarshal_InstanceVar(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_PackedInt> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_PackedInt(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_PackedInt>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_PackedInt>) -> Self {
        Self::RubyMarshal_PackedInt(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_RubyArray> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_RubyArray(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_RubyArray>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_RubyArray>) -> Self {
        Self::RubyMarshal_RubyArray(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_RubyHash> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_RubyHash(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_RubyHash>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_RubyHash>) -> Self {
        Self::RubyMarshal_RubyHash(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_RubyString> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_RubyString(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_RubyString>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_RubyString>) -> Self {
        Self::RubyMarshal_RubyString(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_RubyStruct> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_RubyStruct(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_RubyStruct>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_RubyStruct>) -> Self {
        Self::RubyMarshal_RubyStruct(v)
    }
}
impl TryFrom<&RubyMarshal_Record_Body> for OptRc<RubyMarshal_RubySymbol> {
    type Error = KError;
    fn try_from(v: &RubyMarshal_Record_Body) -> Result<Self, Self::Error> {
        if let RubyMarshal_Record_Body::RubyMarshal_RubySymbol(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<RubyMarshal_RubySymbol>> for RubyMarshal_Record_Body {
    fn from(v: OptRc<RubyMarshal_RubySymbol>) -> Self {
        Self::RubyMarshal_RubySymbol(v)
    }
}
impl KStruct for RubyMarshal_Record {
    type Root = RubyMarshal;
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
        *self_rc.code.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        match *self_rc.code() {
            RubyMarshal_Codes::Bignum => {
                let t = Self::read_into::<_, RubyMarshal_Bignum>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::InstanceVar => {
                let t = Self::read_into::<_, RubyMarshal_InstanceVar>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::PackedInt => {
                let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubyArray => {
                let t = Self::read_into::<_, RubyMarshal_RubyArray>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubyHash => {
                let t = Self::read_into::<_, RubyMarshal_RubyHash>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubyObjectLink => {
                let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubyString => {
                let t = Self::read_into::<_, RubyMarshal_RubyString>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubyStruct => {
                let t = Self::read_into::<_, RubyMarshal_RubyStruct>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubySymbol => {
                let t = Self::read_into::<_, RubyMarshal_RubySymbol>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            RubyMarshal_Codes::RubySymbolLink => {
                let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_Record {
}
impl RubyMarshal_Record {
    pub fn code(&self) -> Ref<'_, RubyMarshal_Codes> {
        self.code.borrow()
    }
}
impl RubyMarshal_Record {
    pub fn body(&self) -> Ref<'_, Option<RubyMarshal_Record_Body>> {
        self.body.borrow()
    }
}
impl RubyMarshal_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_RubyArray {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    num_elements: RefCell<OptRc<RubyMarshal_PackedInt>>,
    elements: RefCell<Vec<OptRc<RubyMarshal_Record>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RubyMarshal_RubyArray {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.num_elements.borrow_mut() = t;
        *self_rc.elements.borrow_mut() = Vec::new();
        let l_elements = usize::try_from(*self_rc.num_elements().value()?)?;
        for _i in 0_usize..l_elements {
            let t = Self::read_into::<_, RubyMarshal_Record>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.elements.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_RubyArray {
}
impl RubyMarshal_RubyArray {
    pub fn num_elements(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.num_elements.borrow()
    }
}
impl RubyMarshal_RubyArray {
    pub fn elements(&self) -> Ref<'_, Vec<OptRc<RubyMarshal_Record>>> {
        self.elements.borrow()
    }
}
impl RubyMarshal_RubyArray {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Hash+and+Hash+with+Default+Value> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_RubyHash {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    num_pairs: RefCell<OptRc<RubyMarshal_PackedInt>>,
    pairs: RefCell<Vec<OptRc<RubyMarshal_Pair>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RubyMarshal_RubyHash {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.num_pairs.borrow_mut() = t;
        *self_rc.pairs.borrow_mut() = Vec::new();
        let l_pairs = usize::try_from(*self_rc.num_pairs().value()?)?;
        for _i in 0_usize..l_pairs {
            let t = Self::read_into::<_, RubyMarshal_Pair>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.pairs.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_RubyHash {
}
impl RubyMarshal_RubyHash {
    pub fn num_pairs(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.num_pairs.borrow()
    }
}
impl RubyMarshal_RubyHash {
    pub fn pairs(&self) -> Ref<'_, Vec<OptRc<RubyMarshal_Pair>>> {
        self.pairs.borrow()
    }
}
impl RubyMarshal_RubyHash {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-String> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_RubyString {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<OptRc<RubyMarshal_PackedInt>>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
impl KStruct for RubyMarshal_RubyString {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.len.borrow_mut() = t;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len().value()?)?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_RubyString {
}
impl RubyMarshal_RubyString {
    pub fn len(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.len.borrow()
    }
}
impl RubyMarshal_RubyString {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl RubyMarshal_RubyString {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl RubyMarshal_RubyString {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

/**
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Struct> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_RubyStruct {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<OptRc<RubyMarshal_Record>>,
    num_members: RefCell<OptRc<RubyMarshal_PackedInt>>,
    members: RefCell<Vec<OptRc<RubyMarshal_Pair>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RubyMarshal_RubyStruct {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        let t = Self::read_into::<_, RubyMarshal_Record>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.name.borrow_mut() = t;
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.num_members.borrow_mut() = t;
        *self_rc.members.borrow_mut() = Vec::new();
        let l_members = usize::try_from(*self_rc.num_members().value()?)?;
        for _i in 0_usize..l_members {
            let t = Self::read_into::<_, RubyMarshal_Pair>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.members.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_RubyStruct {
}

/**
 * Symbol containing the name of the struct.
 */
impl RubyMarshal_RubyStruct {
    pub fn name(&self) -> Ref<'_, OptRc<RubyMarshal_Record>> {
        self.name.borrow()
    }
}

/**
 * Number of members in a struct
 */
impl RubyMarshal_RubyStruct {
    pub fn num_members(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.num_members.borrow()
    }
}
impl RubyMarshal_RubyStruct {
    pub fn members(&self) -> Ref<'_, Vec<OptRc<RubyMarshal_Pair>>> {
        self.members.borrow()
    }
}
impl RubyMarshal_RubyStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://docs.ruby-lang.org/en/2.4.0/marshal_rdoc.html#label-Symbols+and+Byte+Sequence> Source
 */

#[derive(Default, Debug, Clone)]
pub struct RubyMarshal_RubySymbol {
    pub(crate) _root: SharedType<RubyMarshal>,
    pub(crate) _parent: SharedType<RubyMarshal_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<OptRc<RubyMarshal_PackedInt>>,
    name: RefCell<String>,
    _io: RefCell<BytesReader>,
    name_raw: RefCell<Vec<u8>>,
}
impl KStruct for RubyMarshal_RubySymbol {
    type Root = RubyMarshal;
    type Parent = RubyMarshal_Record;

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
        let t = Self::read_into::<_, RubyMarshal_PackedInt>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.len.borrow_mut() = t;
        *self_rc.name.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.len().value()?)?)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RubyMarshal_RubySymbol {
}
impl RubyMarshal_RubySymbol {
    pub fn len(&self) -> Ref<'_, OptRc<RubyMarshal_PackedInt>> {
        self.len.borrow()
    }
}
impl RubyMarshal_RubySymbol {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl RubyMarshal_RubySymbol {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl RubyMarshal_RubySymbol {
    pub fn name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.name_raw.borrow()
    }
}
