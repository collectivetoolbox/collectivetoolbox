// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * .dbf is a relational database format introduced in DOS database
 * management system dBASE in 1982.
 *
 * One .dbf file corresponds to one table and contains a series of headers,
 * specification of fields, and a number of fixed-size records.
 * \sa <http://www.dbase.com/Knowledgebase/INT/db7_file_fmt.htm> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Dbf {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf>,
    pub(crate) _self_shared: SharedType<Self>,
    header1: RefCell<OptRc<Dbf_Header1>>,
    header2: RefCell<OptRc<Dbf_Header2>>,
    header_terminator: RefCell<Vec<u8>>,
    records: RefCell<Vec<OptRc<Dbf_Record>>>,
    _io: RefCell<BytesReader>,
    header2_raw: RefCell<Vec<u8>>,
    records_raw: RefCell<Vec<u8>>,
}
impl KStruct for Dbf {
    type Root = Dbf;
    type Parent = Dbf;

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
        let t = Self::read_into::<_, Dbf_Header1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header1.borrow_mut() = t;
        let _raw_header2 = _io.read_bytes(usize::try_from(((i32::from(*self_rc.header1().len_header())).saturating_sub(12_i32)).saturating_sub(1_i32))?)?;
        *self_rc.header2_raw.borrow_mut() = _raw_header2.clone();
        let _io_header2 = BytesReader::from(_raw_header2);
        let t = Self::read_into::<BytesReader, Dbf_Header2>(&_io_header2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header2.borrow_mut() = t;
        *self_rc.header_terminator.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.header_terminator() == vec![0xdu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/2".to_string() }));
        }
        *self_rc.records.borrow_mut() = Vec::new();
        let l_records = usize::try_from(*self_rc.header1().num_records())?;
        for _i in 0_usize..l_records {
            let _raw_records = _io.read_bytes(usize::from(*self_rc.header1().len_record()))?;
            let _io_records = BytesReader::from(_raw_records);
            let t = Self::read_into::<BytesReader, Dbf_Record>(&_io_records, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.records.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf {
}
impl Dbf {
    pub fn header1(&self) -> Ref<'_, OptRc<Dbf_Header1>> {
        self.header1.borrow()
    }
}
impl Dbf {
    pub fn header2(&self) -> Ref<'_, OptRc<Dbf_Header2>> {
        self.header2.borrow()
    }
}
impl Dbf {
    pub fn header_terminator(&self) -> Ref<'_, Vec<u8>> {
        self.header_terminator.borrow()
    }
}
impl Dbf {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<Dbf_Record>>> {
        self.records.borrow()
    }
}
impl Dbf {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Dbf {
    pub fn header2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.header2_raw.borrow()
    }
}
impl Dbf {
    pub fn records_raw(&self) -> Ref<'_, Vec<u8>> {
        self.records_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Dbf_DeleteState {
    False,
    True,
    Unknown(i64),
}

impl TryFrom<i64> for Dbf_DeleteState {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Dbf_DeleteState> {
        match flag {
            32 => Ok(Dbf_DeleteState::False),
            42 => Ok(Dbf_DeleteState::True),
            _ => Ok(Dbf_DeleteState::Unknown(flag)),
        }
    }
}

impl From<&Dbf_DeleteState> for i64 {
    fn from(v: &Dbf_DeleteState) -> Self {
        match *v {
            Dbf_DeleteState::False => 32,
            Dbf_DeleteState::True => 42,
            Dbf_DeleteState::Unknown(v) => v
        }
    }
}

impl Default for Dbf_DeleteState {
    fn default() -> Self { Dbf_DeleteState::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Dbf_Field {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf_Header2>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    datatype: RefCell<u8>,
    data_address: RefCell<u32>,
    length: RefCell<u8>,
    decimal_count: RefCell<u8>,
    reserved1: RefCell<Vec<u8>>,
    work_area_id: RefCell<u8>,
    reserved2: RefCell<Vec<u8>>,
    set_fields_flag: RefCell<u8>,
    reserved3: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    name_raw: RefCell<Vec<u8>>,
    reserved1_raw: RefCell<Vec<u8>>,
    reserved2_raw: RefCell<Vec<u8>>,
    reserved3_raw: RefCell<Vec<u8>>,
}
impl KStruct for Dbf_Field {
    type Root = Dbf;
    type Parent = Dbf_Header2;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(11_usize)?, Some(0), false, None), "ASCII")?;
        *self_rc.datatype.borrow_mut() = _io.read_u1()?;
        *self_rc.data_address.borrow_mut() = _io.read_u4le()?;
        *self_rc.length.borrow_mut() = _io.read_u1()?;
        *self_rc.decimal_count.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved1.borrow_mut() = _io.read_bytes(2_usize)?;
        *self_rc.work_area_id.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved2.borrow_mut() = _io.read_bytes(2_usize)?;
        *self_rc.set_fields_flag.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved3.borrow_mut() = _io.read_bytes(8_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf_Field {
}
impl Dbf_Field {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl Dbf_Field {
    pub fn datatype(&self) -> Ref<'_, u8> {
        self.datatype.borrow()
    }
}
impl Dbf_Field {
    pub fn data_address(&self) -> Ref<'_, u32> {
        self.data_address.borrow()
    }
}
impl Dbf_Field {
    pub fn length(&self) -> Ref<'_, u8> {
        self.length.borrow()
    }
}
impl Dbf_Field {
    pub fn decimal_count(&self) -> Ref<'_, u8> {
        self.decimal_count.borrow()
    }
}
impl Dbf_Field {
    pub fn reserved1(&self) -> Ref<'_, Vec<u8>> {
        self.reserved1.borrow()
    }
}
impl Dbf_Field {
    pub fn work_area_id(&self) -> Ref<'_, u8> {
        self.work_area_id.borrow()
    }
}
impl Dbf_Field {
    pub fn reserved2(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2.borrow()
    }
}
impl Dbf_Field {
    pub fn set_fields_flag(&self) -> Ref<'_, u8> {
        self.set_fields_flag.borrow()
    }
}
impl Dbf_Field {
    pub fn reserved3(&self) -> Ref<'_, Vec<u8>> {
        self.reserved3.borrow()
    }
}
impl Dbf_Field {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Dbf_Field {
    pub fn name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.name_raw.borrow()
    }
}
impl Dbf_Field {
    pub fn reserved1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved1_raw.borrow()
    }
}
impl Dbf_Field {
    pub fn reserved2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2_raw.borrow()
    }
}
impl Dbf_Field {
    pub fn reserved3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved3_raw.borrow()
    }
}

/**
 * \sa http://www.dbase.com/Knowledgebase/INT/db7_file_fmt.htm - section 1.1
 */

#[derive(Default, Debug, Clone)]
pub struct Dbf_Header1 {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u8>,
    last_update_y: RefCell<u8>,
    last_update_m: RefCell<u8>,
    last_update_d: RefCell<u8>,
    num_records: RefCell<u32>,
    len_header: RefCell<u16>,
    len_record: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_dbase_level: Cell<bool>,
    dbase_level: RefCell<i32>,
}
impl KStruct for Dbf_Header1 {
    type Root = Dbf;
    type Parent = Dbf;

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
        *self_rc.version.borrow_mut() = _io.read_u1()?;
        *self_rc.last_update_y.borrow_mut() = _io.read_u1()?;
        *self_rc.last_update_m.borrow_mut() = _io.read_u1()?;
        *self_rc.last_update_d.borrow_mut() = _io.read_u1()?;
        *self_rc.num_records.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_header.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_record.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf_Header1 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn dbase_level(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_dbase_level.get() {
            return Ok(self.dbase_level.borrow());
        }
        self.f_dbase_level.set(true);
        *self.dbase_level.borrow_mut() = (((i32::from(*self.version())) & (7_i32))).try_into()?;
        Ok(self.dbase_level.borrow())
    }
}
impl Dbf_Header1 {
    pub fn version(&self) -> Ref<'_, u8> {
        self.version.borrow()
    }
}
impl Dbf_Header1 {
    pub fn last_update_y(&self) -> Ref<'_, u8> {
        self.last_update_y.borrow()
    }
}
impl Dbf_Header1 {
    pub fn last_update_m(&self) -> Ref<'_, u8> {
        self.last_update_m.borrow()
    }
}
impl Dbf_Header1 {
    pub fn last_update_d(&self) -> Ref<'_, u8> {
        self.last_update_d.borrow()
    }
}
impl Dbf_Header1 {
    pub fn num_records(&self) -> Ref<'_, u32> {
        self.num_records.borrow()
    }
}
impl Dbf_Header1 {
    pub fn len_header(&self) -> Ref<'_, u16> {
        self.len_header.borrow()
    }
}
impl Dbf_Header1 {
    pub fn len_record(&self) -> Ref<'_, u16> {
        self.len_record.borrow()
    }
}
impl Dbf_Header1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dbf_Header2 {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf>,
    pub(crate) _self_shared: SharedType<Self>,
    header_dbase_3: RefCell<OptRc<Dbf_HeaderDbase3>>,
    header_dbase_7: RefCell<OptRc<Dbf_HeaderDbase7>>,
    fields: RefCell<Vec<OptRc<Dbf_Field>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Dbf_Header2 {
    type Root = Dbf;
    type Parent = Dbf;

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
        if *self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header1().dbase_level()? == 3 {
            let t = Self::read_into::<_, Dbf_HeaderDbase3>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.header_dbase_3.borrow_mut() = t;
        }
        if *self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header1().dbase_level()? == 7 {
            let t = Self::read_into::<_, Dbf_HeaderDbase7>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.header_dbase_7.borrow_mut() = t;
        }
        *self_rc.fields.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Dbf_Field>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.fields.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf_Header2 {
}
impl Dbf_Header2 {
    pub fn header_dbase_3(&self) -> Ref<'_, OptRc<Dbf_HeaderDbase3>> {
        self.header_dbase_3.borrow()
    }
}
impl Dbf_Header2 {
    pub fn header_dbase_7(&self) -> Ref<'_, OptRc<Dbf_HeaderDbase7>> {
        self.header_dbase_7.borrow()
    }
}
impl Dbf_Header2 {
    pub fn fields(&self) -> Ref<'_, Vec<OptRc<Dbf_Field>>> {
        self.fields.borrow()
    }
}
impl Dbf_Header2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dbf_HeaderDbase3 {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf_Header2>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<Vec<u8>>,
    reserved2: RefCell<Vec<u8>>,
    reserved3: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    reserved1_raw: RefCell<Vec<u8>>,
    reserved2_raw: RefCell<Vec<u8>>,
    reserved3_raw: RefCell<Vec<u8>>,
}
impl KStruct for Dbf_HeaderDbase3 {
    type Root = Dbf;
    type Parent = Dbf_Header2;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bytes(3_usize)?;
        *self_rc.reserved2.borrow_mut() = _io.read_bytes(13_usize)?;
        *self_rc.reserved3.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf_HeaderDbase3 {
}
impl Dbf_HeaderDbase3 {
    pub fn reserved1(&self) -> Ref<'_, Vec<u8>> {
        self.reserved1.borrow()
    }
}
impl Dbf_HeaderDbase3 {
    pub fn reserved2(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2.borrow()
    }
}
impl Dbf_HeaderDbase3 {
    pub fn reserved3(&self) -> Ref<'_, Vec<u8>> {
        self.reserved3.borrow()
    }
}
impl Dbf_HeaderDbase3 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Dbf_HeaderDbase3 {
    pub fn reserved1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved1_raw.borrow()
    }
}
impl Dbf_HeaderDbase3 {
    pub fn reserved2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2_raw.borrow()
    }
}
impl Dbf_HeaderDbase3 {
    pub fn reserved3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved3_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dbf_HeaderDbase7 {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf_Header2>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<Vec<u8>>,
    has_incomplete_transaction: RefCell<u8>,
    dbase_iv_encryption: RefCell<u8>,
    reserved2: RefCell<Vec<u8>>,
    production_mdx: RefCell<u8>,
    language_driver_id: RefCell<u8>,
    reserved3: RefCell<Vec<u8>>,
    language_driver_name: RefCell<Vec<u8>>,
    reserved4: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    reserved2_raw: RefCell<Vec<u8>>,
    language_driver_name_raw: RefCell<Vec<u8>>,
    reserved4_raw: RefCell<Vec<u8>>,
}
impl KStruct for Dbf_HeaderDbase7 {
    type Root = Dbf;
    type Parent = Dbf_Header2;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.reserved1() == vec![0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header_dbase_7/seq/0".to_string() }));
        }
        *self_rc.has_incomplete_transaction.borrow_mut() = _io.read_u1()?;
        *self_rc.dbase_iv_encryption.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved2.borrow_mut() = _io.read_bytes(12_usize)?;
        *self_rc.production_mdx.borrow_mut() = _io.read_u1()?;
        *self_rc.language_driver_id.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved3.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.reserved3() == vec![0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header_dbase_7/seq/6".to_string() }));
        }
        *self_rc.language_driver_name.borrow_mut() = _io.read_bytes(32_usize)?;
        *self_rc.reserved4.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf_HeaderDbase7 {
}
impl Dbf_HeaderDbase7 {
    pub fn reserved1(&self) -> Ref<'_, Vec<u8>> {
        self.reserved1.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn has_incomplete_transaction(&self) -> Ref<'_, u8> {
        self.has_incomplete_transaction.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn dbase_iv_encryption(&self) -> Ref<'_, u8> {
        self.dbase_iv_encryption.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn reserved2(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn production_mdx(&self) -> Ref<'_, u8> {
        self.production_mdx.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn language_driver_id(&self) -> Ref<'_, u8> {
        self.language_driver_id.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn reserved3(&self) -> Ref<'_, Vec<u8>> {
        self.reserved3.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn language_driver_name(&self) -> Ref<'_, Vec<u8>> {
        self.language_driver_name.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn reserved4(&self) -> Ref<'_, Vec<u8>> {
        self.reserved4.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn reserved2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2_raw.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn language_driver_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.language_driver_name_raw.borrow()
    }
}
impl Dbf_HeaderDbase7 {
    pub fn reserved4_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved4_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dbf_Record {
    pub(crate) _root: SharedType<Dbf>,
    pub(crate) _parent: SharedType<Dbf>,
    pub(crate) _self_shared: SharedType<Self>,
    deleted: RefCell<Dbf_DeleteState>,
    record_fields: RefCell<Vec<Vec<u8>>>,
    _io: RefCell<BytesReader>,
    record_fields_raw: RefCell<Vec<u8>>,
}
impl KStruct for Dbf_Record {
    type Root = Dbf;
    type Parent = Dbf;

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
        *self_rc.deleted.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.record_fields.borrow_mut() = Vec::new();
        let l_record_fields = usize::try_from(self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header2().fields().len())?;
        for _i in 0_usize..l_record_fields {
            self_rc.record_fields.borrow_mut().push(_io.read_bytes(usize::from(*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header2().fields().get(_i).ok_or(KError::CastError)?.length()))?);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Dbf_Record {
}
impl Dbf_Record {
    pub fn deleted(&self) -> Ref<'_, Dbf_DeleteState> {
        self.deleted.borrow()
    }
}
impl Dbf_Record {
    pub fn record_fields(&self) -> Ref<'_, Vec<Vec<u8>>> {
        self.record_fields.borrow()
    }
}
impl Dbf_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Dbf_Record {
    pub fn record_fields_raw(&self) -> Ref<'_, Vec<u8>> {
        self.record_fields_raw.borrow()
    }
}
