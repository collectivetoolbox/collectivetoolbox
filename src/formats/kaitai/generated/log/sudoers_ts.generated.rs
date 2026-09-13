// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * This spec can be used to parse sudo time stamp files located in directories
 * such as /run/sudo/ts/$USER or /var/lib/sudo/ts/$USER.
 * \sa <https://www.sudo.ws/docs/man/1.8.27/sudoers_timestamp.man/> Source
 */

#[derive(Default, Debug, Clone)]
pub struct SudoersTs {
    pub(crate) _root: SharedType<SudoersTs>,
    pub(crate) _parent: SharedType<SudoersTs>,
    pub(crate) _self_shared: SharedType<Self>,
    records: RefCell<Vec<OptRc<SudoersTs_Record>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SudoersTs {
    type Root = SudoersTs;
    type Parent = SudoersTs;

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
        *self_rc.records.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, SudoersTs_Record>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl SudoersTs {
}
impl SudoersTs {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<SudoersTs_Record>>> {
        self.records.borrow()
    }
}
impl SudoersTs {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum SudoersTs_TsType {
    Global,
    Tty,
    Ppid,
    Lockexcl,
    Unknown(i64),
}

impl TryFrom<i64> for SudoersTs_TsType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<SudoersTs_TsType> {
        match flag {
            1 => Ok(SudoersTs_TsType::Global),
            2 => Ok(SudoersTs_TsType::Tty),
            3 => Ok(SudoersTs_TsType::Ppid),
            4 => Ok(SudoersTs_TsType::Lockexcl),
            _ => Ok(SudoersTs_TsType::Unknown(flag)),
        }
    }
}

impl From<&SudoersTs_TsType> for i64 {
    fn from(v: &SudoersTs_TsType) -> Self {
        match *v {
            SudoersTs_TsType::Global => 1,
            SudoersTs_TsType::Tty => 2,
            SudoersTs_TsType::Ppid => 3,
            SudoersTs_TsType::Lockexcl => 4,
            SudoersTs_TsType::Unknown(v) => v
        }
    }
}

impl Default for SudoersTs_TsType {
    fn default() -> Self { SudoersTs_TsType::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct SudoersTs_Record {
    pub(crate) _root: SharedType<SudoersTs>,
    pub(crate) _parent: SharedType<SudoersTs>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u16>,
    len_record: RefCell<u16>,
    payload: RefCell<Option<SudoersTs_Record_Payload>>,
    _io: RefCell<BytesReader>,
    payload_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SudoersTs_Record_Payload {
    SudoersTs_RecordV1(OptRc<SudoersTs_RecordV1>),
    SudoersTs_RecordV2(OptRc<SudoersTs_RecordV2>),
    Bytes(Vec<u8>),
}
impl From<&SudoersTs_Record_Payload> for OptRc<SudoersTs_RecordV1> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &SudoersTs_Record_Payload) -> Self {
        if let SudoersTs_Record_Payload::SudoersTs_RecordV1(x) = v {
            return x.clone();
        }
        panic!("expected SudoersTs_Record_Payload::SudoersTs_RecordV1, got {:?}", v)
    }
}
impl From<OptRc<SudoersTs_RecordV1>> for SudoersTs_Record_Payload {
    fn from(v: OptRc<SudoersTs_RecordV1>) -> Self {
        Self::SudoersTs_RecordV1(v)
    }
}
impl From<&SudoersTs_Record_Payload> for OptRc<SudoersTs_RecordV2> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &SudoersTs_Record_Payload) -> Self {
        if let SudoersTs_Record_Payload::SudoersTs_RecordV2(x) = v {
            return x.clone();
        }
        panic!("expected SudoersTs_Record_Payload::SudoersTs_RecordV2, got {:?}", v)
    }
}
impl From<OptRc<SudoersTs_RecordV2>> for SudoersTs_Record_Payload {
    fn from(v: OptRc<SudoersTs_RecordV2>) -> Self {
        Self::SudoersTs_RecordV2(v)
    }
}
impl From<&SudoersTs_Record_Payload> for Vec<u8> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &SudoersTs_Record_Payload) -> Self {
        if let SudoersTs_Record_Payload::Bytes(x) = v {
            return x.clone();
        }
        panic!("expected SudoersTs_Record_Payload::Bytes, got {:?}", v)
    }
}
impl From<Vec<u8>> for SudoersTs_Record_Payload {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for SudoersTs_Record {
    type Root = SudoersTs;
    type Parent = SudoersTs;

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
        *self_rc.version.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_record.borrow_mut() = _io.read_u2le()?;
        match *self_rc.version() {
            1 => {
                *self_rc.payload_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let payload_raw = self_rc.payload_raw.borrow();
                let _t_payload_raw_io = BytesReader::from(payload_raw.clone());
                let t = Self::read_into::<BytesReader, SudoersTs_RecordV1>(&_t_payload_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.payload.borrow_mut() = Some(t);
            }
            2 => {
                *self_rc.payload_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let payload_raw = self_rc.payload_raw.borrow();
                let _t_payload_raw_io = BytesReader::from(payload_raw.clone());
                let t = Self::read_into::<BytesReader, SudoersTs_RecordV2>(&_t_payload_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.payload.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.payload.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        Ok(())
    }
}
impl SudoersTs_Record {
}

/**
 * version number of the timestamp_entry struct
 */
impl SudoersTs_Record {
    pub fn version(&self) -> Ref<'_, u16> {
        self.version.borrow()
    }
}

/**
 * size of the record in bytes
 */
impl SudoersTs_Record {
    pub fn len_record(&self) -> Ref<'_, u16> {
        self.len_record.borrow()
    }
}
impl SudoersTs_Record {
    pub fn payload(&self) -> Ref<'_, Option<SudoersTs_Record_Payload>> {
        self.payload.borrow()
    }
}
impl SudoersTs_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SudoersTs_Record {
    pub fn payload_raw(&self) -> Ref<'_, Vec<u8>> {
        self.payload_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SudoersTs_RecordV1 {
    pub(crate) _root: SharedType<SudoersTs>,
    pub(crate) _parent: SharedType<SudoersTs_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    r#type: RefCell<SudoersTs_TsType>,
    flags: RefCell<OptRc<SudoersTs_TsFlag>>,
    auth_uid: RefCell<u32>,
    sid: RefCell<u32>,
    ts: RefCell<OptRc<SudoersTs_Timespec>>,
    ttydev: RefCell<u32>,
    ppid: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SudoersTs_RecordV1 {
    type Root = SudoersTs;
    type Parent = SudoersTs_Record;

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
        *self_rc.r#type.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        let t = Self::read_into::<_, SudoersTs_TsFlag>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.flags.borrow_mut() = t;
        *self_rc.auth_uid.borrow_mut() = _io.read_u4le()?;
        *self_rc.sid.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, SudoersTs_Timespec>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.ts.borrow_mut() = t;
        if *self_rc.r#type() == SudoersTs_TsType::Tty {
            *self_rc.ttydev.borrow_mut() = _io.read_u4le()?;
        }
        if *self_rc.r#type() == SudoersTs_TsType::Ppid {
            *self_rc.ppid.borrow_mut() = _io.read_u4le()?;
        }
        Ok(())
    }
}
impl SudoersTs_RecordV1 {
}

/**
 * record type
 */
impl SudoersTs_RecordV1 {
    pub fn r#type(&self) -> Ref<'_, SudoersTs_TsType> {
        self.r#type.borrow()
    }
}

/**
 * record flags
 */
impl SudoersTs_RecordV1 {
    pub fn flags(&self) -> Ref<'_, OptRc<SudoersTs_TsFlag>> {
        self.flags.borrow()
    }
}

/**
 * user ID that was used for authentication
 */
impl SudoersTs_RecordV1 {
    pub fn auth_uid(&self) -> Ref<'_, u32> {
        self.auth_uid.borrow()
    }
}

/**
 * session ID associated with tty/ppid
 */
impl SudoersTs_RecordV1 {
    pub fn sid(&self) -> Ref<'_, u32> {
        self.sid.borrow()
    }
}

/**
 * time stamp, from a monotonic time source
 */
impl SudoersTs_RecordV1 {
    pub fn ts(&self) -> Ref<'_, OptRc<SudoersTs_Timespec>> {
        self.ts.borrow()
    }
}

/**
 * device number of the terminal associated with the session
 */
impl SudoersTs_RecordV1 {
    pub fn ttydev(&self) -> Ref<'_, u32> {
        self.ttydev.borrow()
    }
}

/**
 * ID of the parent process
 */
impl SudoersTs_RecordV1 {
    pub fn ppid(&self) -> Ref<'_, u32> {
        self.ppid.borrow()
    }
}
impl SudoersTs_RecordV1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SudoersTs_RecordV2 {
    pub(crate) _root: SharedType<SudoersTs>,
    pub(crate) _parent: SharedType<SudoersTs_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    r#type: RefCell<SudoersTs_TsType>,
    flags: RefCell<OptRc<SudoersTs_TsFlag>>,
    auth_uid: RefCell<u32>,
    sid: RefCell<u32>,
    start_time: RefCell<OptRc<SudoersTs_Timespec>>,
    ts: RefCell<OptRc<SudoersTs_Timespec>>,
    ttydev: RefCell<u32>,
    ppid: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SudoersTs_RecordV2 {
    type Root = SudoersTs;
    type Parent = SudoersTs_Record;

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
        *self_rc.r#type.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        let t = Self::read_into::<_, SudoersTs_TsFlag>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.flags.borrow_mut() = t;
        *self_rc.auth_uid.borrow_mut() = _io.read_u4le()?;
        *self_rc.sid.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, SudoersTs_Timespec>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.start_time.borrow_mut() = t;
        let t = Self::read_into::<_, SudoersTs_Timespec>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.ts.borrow_mut() = t;
        if *self_rc.r#type() == SudoersTs_TsType::Tty {
            *self_rc.ttydev.borrow_mut() = _io.read_u4le()?;
        }
        if *self_rc.r#type() == SudoersTs_TsType::Ppid {
            *self_rc.ppid.borrow_mut() = _io.read_u4le()?;
        }
        Ok(())
    }
}
impl SudoersTs_RecordV2 {
}

/**
 * record type
 */
impl SudoersTs_RecordV2 {
    pub fn r#type(&self) -> Ref<'_, SudoersTs_TsType> {
        self.r#type.borrow()
    }
}

/**
 * record flags
 */
impl SudoersTs_RecordV2 {
    pub fn flags(&self) -> Ref<'_, OptRc<SudoersTs_TsFlag>> {
        self.flags.borrow()
    }
}

/**
 * user ID that was used for authentication
 */
impl SudoersTs_RecordV2 {
    pub fn auth_uid(&self) -> Ref<'_, u32> {
        self.auth_uid.borrow()
    }
}

/**
 * ID of the user's terminal session, if present (when type is TS_TTY)
 */
impl SudoersTs_RecordV2 {
    pub fn sid(&self) -> Ref<'_, u32> {
        self.sid.borrow()
    }
}

/**
 * start time of the session leader for records of type TS_TTY or of the parent process for records of type TS_PPID
 */
impl SudoersTs_RecordV2 {
    pub fn start_time(&self) -> Ref<'_, OptRc<SudoersTs_Timespec>> {
        self.start_time.borrow()
    }
}

/**
 * actual time stamp, from a monotonic time source
 */
impl SudoersTs_RecordV2 {
    pub fn ts(&self) -> Ref<'_, OptRc<SudoersTs_Timespec>> {
        self.ts.borrow()
    }
}

/**
 * device number of the terminal associated with the session
 */
impl SudoersTs_RecordV2 {
    pub fn ttydev(&self) -> Ref<'_, u32> {
        self.ttydev.borrow()
    }
}

/**
 * ID of the parent process
 */
impl SudoersTs_RecordV2 {
    pub fn ppid(&self) -> Ref<'_, u32> {
        self.ppid.borrow()
    }
}
impl SudoersTs_RecordV2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SudoersTs_Timespec {
    pub(crate) _root: SharedType<SudoersTs>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    sec: RefCell<i64>,
    nsec: RefCell<i64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SudoersTs_Timespec {
    type Root = SudoersTs;
    type Parent = KStructUnit;

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
        *self_rc.sec.borrow_mut() = _io.read_s8le()?;
        *self_rc.nsec.borrow_mut() = _io.read_s8le()?;
        Ok(())
    }
}
impl SudoersTs_Timespec {
}

/**
 * seconds
 */
impl SudoersTs_Timespec {
    pub fn sec(&self) -> Ref<'_, i64> {
        self.sec.borrow()
    }
}

/**
 * nanoseconds
 */
impl SudoersTs_Timespec {
    pub fn nsec(&self) -> Ref<'_, i64> {
        self.nsec.borrow()
    }
}
impl SudoersTs_Timespec {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SudoersTs_TsFlag {
    pub(crate) _root: SharedType<SudoersTs>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved0: RefCell<u64>,
    anyuid: RefCell<bool>,
    disabled: RefCell<bool>,
    reserved1: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SudoersTs_TsFlag {
    type Root = SudoersTs;
    type Parent = KStructUnit;

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
        *self_rc.reserved0.borrow_mut() = _io.read_bits_int_be(6)?;
        *self_rc.anyuid.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.disabled.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(8)?;
        Ok(())
    }
}
impl SudoersTs_TsFlag {
}

/**
 * Reserved (unused) bits
 */
impl SudoersTs_TsFlag {
    pub fn reserved0(&self) -> Ref<'_, u64> {
        self.reserved0.borrow()
    }
}

/**
 * ignore uid
 */
impl SudoersTs_TsFlag {
    pub fn anyuid(&self) -> Ref<'_, bool> {
        self.anyuid.borrow()
    }
}

/**
 * entry disabled
 */
impl SudoersTs_TsFlag {
    pub fn disabled(&self) -> Ref<'_, bool> {
        self.disabled.borrow()
    }
}

/**
 * Reserved (unused) bits
 */
impl SudoersTs_TsFlag {
    pub fn reserved1(&self) -> Ref<'_, u64> {
        self.reserved1.borrow()
    }
}
impl SudoersTs_TsFlag {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
