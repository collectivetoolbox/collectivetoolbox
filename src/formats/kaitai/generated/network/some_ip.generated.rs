// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::some_ip_sd::SomeIpSd;

/**
 * SOME/IP (Scalable service-Oriented MiddlewarE over IP) is an automotive/embedded
 * communication protocol which supports remoteprocedure calls, event notifications
 * and the underlying serialization/wire format.
 * \sa <https://www.autosar.org/fileadmin/standards/foundation/19-11/AUTOSAR_PRS_SOMEIPProtocol.pdf> Source
 */

#[derive(Default, Debug, Clone)]
pub struct SomeIp {
    pub(crate) _root: SharedType<SomeIp>,
    pub(crate) _parent: SharedType<SomeIp>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<SomeIp_Header>>,
    payload: RefCell<Option<SomeIp_Payload>>,
    _io: RefCell<BytesReader>,
    payload_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SomeIp_Payload {
    SomeIpSd(OptRc<SomeIpSd>),
    Bytes(Vec<u8>),
}
impl TryFrom<&SomeIp_Payload> for OptRc<SomeIpSd> {
    type Error = KError;
    fn try_from(v: &SomeIp_Payload) -> Result<Self, Self::Error> {
        if let SomeIp_Payload::SomeIpSd(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SomeIpSd>> for SomeIp_Payload {
    fn from(v: OptRc<SomeIpSd>) -> Self {
        Self::SomeIpSd(v)
    }
}
impl TryFrom<&SomeIp_Payload> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &SomeIp_Payload) -> Result<Self, Self::Error> {
        if let SomeIp_Payload::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for SomeIp_Payload {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for SomeIp {
    type Root = SomeIp;
    type Parent = SomeIp;

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
        let t = Self::read_into::<_, SomeIp_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        match *self_rc.header().message_id().value()? {
            4294934784 => {
                *self_rc.payload_raw.borrow_mut() = _io.read_bytes(usize::try_from(((i64::try_from(self_rc.header().len())?)).saturating_sub(8_u32))?)?.into();
                let payload_raw = self_rc.payload_raw.borrow();
                let _t_payload_raw_io = BytesReader::from(payload_raw.clone());
                let t = Self::read_into::<BytesReader, SomeIpSd>(&_t_payload_raw_io, None, None)?.into();
                *self_rc.payload.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.payload.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SomeIp {
}
impl SomeIp {
    pub fn header(&self) -> Ref<'_, OptRc<SomeIp_Header>> {
        self.header.borrow()
    }
}
impl SomeIp {
    pub fn payload(&self) -> Ref<'_, Option<SomeIp_Payload>> {
        self.payload.borrow()
    }
}
impl SomeIp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SomeIp {
    pub fn payload_raw(&self) -> Ref<'_, Vec<u8>> {
        self.payload_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SomeIp_Header {
    pub(crate) _root: SharedType<SomeIp>,
    pub(crate) _parent: SharedType<SomeIp>,
    pub(crate) _self_shared: SharedType<Self>,
    message_id: RefCell<OptRc<SomeIp_Header_MessageId>>,
    length: RefCell<u32>,
    request_id: RefCell<OptRc<SomeIp_Header_RequestId>>,
    protocol_version: RefCell<u8>,
    interface_version: RefCell<u8>,
    message_type: RefCell<SomeIp_Header_MessageTypeEnum>,
    return_code: RefCell<SomeIp_Header_ReturnCodeEnum>,
    _io: RefCell<BytesReader>,
    message_id_raw: RefCell<Vec<u8>>,
    request_id_raw: RefCell<Vec<u8>>,
    f_is_valid_service_discovery: Cell<bool>,
    is_valid_service_discovery: RefCell<bool>,
}
impl KStruct for SomeIp_Header {
    type Root = SomeIp;
    type Parent = SomeIp;

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
        let _raw_message_id = _io.read_bytes(4_usize)?;
        *self_rc.message_id_raw.borrow_mut() = _raw_message_id.clone();
        let _io_message_id = BytesReader::from(_raw_message_id);
        let t = Self::read_into::<BytesReader, SomeIp_Header_MessageId>(&_io_message_id, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.message_id.borrow_mut() = t;
        *self_rc.length.borrow_mut() = _io.read_u4be()?;
        let _raw_request_id = _io.read_bytes(4_usize)?;
        *self_rc.request_id_raw.borrow_mut() = _raw_request_id.clone();
        let _io_request_id = BytesReader::from(_raw_request_id);
        let t = Self::read_into::<BytesReader, SomeIp_Header_RequestId>(&_io_request_id, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.request_id.borrow_mut() = t;
        *self_rc.protocol_version.borrow_mut() = _io.read_u1()?;
        *self_rc.interface_version.borrow_mut() = _io.read_u1()?;
        *self_rc.message_type.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.return_code.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SomeIp_Header {

    /**
     * auxiliary value
     * \sa AUTOSAR_PRS_SOMEIPServiceDiscoveryProtocol.pdf - section 4.1.2.1 General Requirements
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_valid_service_discovery(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_valid_service_discovery.get() {
            return Ok(self.is_valid_service_discovery.borrow());
        }
        self.f_is_valid_service_discovery.set(true);
        *self.is_valid_service_discovery.borrow_mut() = ( ((((to_i128(*self.message_id().value()?)) == (to_i128(4294934784_i64)))) && (((to_i128(*self.protocol_version())) == (to_i128(1)))) && (((to_i128(*self.interface_version())) == (to_i128(1)))) && (*self.message_type() == SomeIp_Header_MessageTypeEnum::Notification) && (*self.return_code() == SomeIp_Header_ReturnCodeEnum::Ok)) ).try_into()?;
        Ok(self.is_valid_service_discovery.borrow())
    }
}

/**
 * The Message ID shall be a 32 Bit identifier that is used to identify
 * the RPC call to a method of an application or to identify an event.
 */
impl SomeIp_Header {
    pub fn message_id(&self) -> Ref<'_, OptRc<SomeIp_Header_MessageId>> {
        self.message_id.borrow()
    }
}

/**
 * [PRS_SOMEIP_00042] Length field shall contain the length in Byte
 * starting from Request ID/Client ID until the end of the SOME/IP message.
 */
impl SomeIp_Header {
    pub fn length(&self) -> Ref<'_, u32> {
        self.length.borrow()
    }
}

/**
 * The Request ID allows a provider and subscriber to differentiate
 * multiple parallel uses of the same method, event, getter or setter.
 */
impl SomeIp_Header {
    pub fn request_id(&self) -> Ref<'_, OptRc<SomeIp_Header_RequestId>> {
        self.request_id.borrow()
    }
}

/**
 * The Protocol Version identifies the used SOME/IP Header format
 * (not including the Payload format).
 */
impl SomeIp_Header {
    pub fn protocol_version(&self) -> Ref<'_, u8> {
        self.protocol_version.borrow()
    }
}

/**
 * Interface Version shall be an 8 Bit field that contains the
 * MajorVersion of the Service Interface.
 */
impl SomeIp_Header {
    pub fn interface_version(&self) -> Ref<'_, u8> {
        self.interface_version.borrow()
    }
}

/**
 * The Message Type field is used to differentiate different types of
 * messages.
 * \sa AUTOSAR_PRS_SOMEIPProtocol.pdf - Table 4.4
 */
impl SomeIp_Header {
    pub fn message_type(&self) -> Ref<'_, SomeIp_Header_MessageTypeEnum> {
        self.message_type.borrow()
    }
}

/**
 * The Return Code shall be used to signal whether a request was
 * successfully processed.
 * \sa AUTOSAR_PRS_SOMEIPProtocol.pdf - Table 4.5
 */
impl SomeIp_Header {
    pub fn return_code(&self) -> Ref<'_, SomeIp_Header_ReturnCodeEnum> {
        self.return_code.borrow()
    }
}
impl SomeIp_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SomeIp_Header {
    pub fn message_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.message_id_raw.borrow()
    }
}
impl SomeIp_Header {
    pub fn request_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.request_id_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SomeIp_Header_MessageTypeEnum {
    Request,
    RequestNoReturn,
    Notification,
    RequestAck,
    RequestNoReturnAck,
    NotificationAck,
    Response,
    Error,
    ResponseAck,
    ErrorAck,
    Unknown(i64),
}

impl TryFrom<i64> for SomeIp_Header_MessageTypeEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<SomeIp_Header_MessageTypeEnum> {
        match flag {
            0 => Ok(SomeIp_Header_MessageTypeEnum::Request),
            1 => Ok(SomeIp_Header_MessageTypeEnum::RequestNoReturn),
            2 => Ok(SomeIp_Header_MessageTypeEnum::Notification),
            64 => Ok(SomeIp_Header_MessageTypeEnum::RequestAck),
            65 => Ok(SomeIp_Header_MessageTypeEnum::RequestNoReturnAck),
            66 => Ok(SomeIp_Header_MessageTypeEnum::NotificationAck),
            128 => Ok(SomeIp_Header_MessageTypeEnum::Response),
            129 => Ok(SomeIp_Header_MessageTypeEnum::Error),
            192 => Ok(SomeIp_Header_MessageTypeEnum::ResponseAck),
            193 => Ok(SomeIp_Header_MessageTypeEnum::ErrorAck),
            _ => Ok(SomeIp_Header_MessageTypeEnum::Unknown(flag)),
        }
    }
}

impl From<&SomeIp_Header_MessageTypeEnum> for i64 {
    fn from(v: &SomeIp_Header_MessageTypeEnum) -> Self {
        match *v {
            SomeIp_Header_MessageTypeEnum::Request => 0,
            SomeIp_Header_MessageTypeEnum::RequestNoReturn => 1,
            SomeIp_Header_MessageTypeEnum::Notification => 2,
            SomeIp_Header_MessageTypeEnum::RequestAck => 64,
            SomeIp_Header_MessageTypeEnum::RequestNoReturnAck => 65,
            SomeIp_Header_MessageTypeEnum::NotificationAck => 66,
            SomeIp_Header_MessageTypeEnum::Response => 128,
            SomeIp_Header_MessageTypeEnum::Error => 129,
            SomeIp_Header_MessageTypeEnum::ResponseAck => 192,
            SomeIp_Header_MessageTypeEnum::ErrorAck => 193,
            SomeIp_Header_MessageTypeEnum::Unknown(v) => v
        }
    }
}

impl Default for SomeIp_Header_MessageTypeEnum {
    fn default() -> Self { SomeIp_Header_MessageTypeEnum::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SomeIp_Header_ReturnCodeEnum {
    Ok,
    NotOk,
    UnknownService,
    UnknownMethod,
    NotReady,
    NotReachable,
    TimeOut,
    WrongProtocolVersion,
    WrongInterfaceVersion,
    MalformedMessage,
    WrongMessageType,
    Unknown(i64),
}

impl TryFrom<i64> for SomeIp_Header_ReturnCodeEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<SomeIp_Header_ReturnCodeEnum> {
        match flag {
            0 => Ok(SomeIp_Header_ReturnCodeEnum::Ok),
            1 => Ok(SomeIp_Header_ReturnCodeEnum::NotOk),
            2 => Ok(SomeIp_Header_ReturnCodeEnum::UnknownService),
            3 => Ok(SomeIp_Header_ReturnCodeEnum::UnknownMethod),
            4 => Ok(SomeIp_Header_ReturnCodeEnum::NotReady),
            5 => Ok(SomeIp_Header_ReturnCodeEnum::NotReachable),
            6 => Ok(SomeIp_Header_ReturnCodeEnum::TimeOut),
            7 => Ok(SomeIp_Header_ReturnCodeEnum::WrongProtocolVersion),
            8 => Ok(SomeIp_Header_ReturnCodeEnum::WrongInterfaceVersion),
            9 => Ok(SomeIp_Header_ReturnCodeEnum::MalformedMessage),
            10 => Ok(SomeIp_Header_ReturnCodeEnum::WrongMessageType),
            _ => Ok(SomeIp_Header_ReturnCodeEnum::Unknown(flag)),
        }
    }
}

impl From<&SomeIp_Header_ReturnCodeEnum> for i64 {
    fn from(v: &SomeIp_Header_ReturnCodeEnum) -> Self {
        match *v {
            SomeIp_Header_ReturnCodeEnum::Ok => 0,
            SomeIp_Header_ReturnCodeEnum::NotOk => 1,
            SomeIp_Header_ReturnCodeEnum::UnknownService => 2,
            SomeIp_Header_ReturnCodeEnum::UnknownMethod => 3,
            SomeIp_Header_ReturnCodeEnum::NotReady => 4,
            SomeIp_Header_ReturnCodeEnum::NotReachable => 5,
            SomeIp_Header_ReturnCodeEnum::TimeOut => 6,
            SomeIp_Header_ReturnCodeEnum::WrongProtocolVersion => 7,
            SomeIp_Header_ReturnCodeEnum::WrongInterfaceVersion => 8,
            SomeIp_Header_ReturnCodeEnum::MalformedMessage => 9,
            SomeIp_Header_ReturnCodeEnum::WrongMessageType => 10,
            SomeIp_Header_ReturnCodeEnum::Unknown(v) => v
        }
    }
}

impl Default for SomeIp_Header_ReturnCodeEnum {
    fn default() -> Self { SomeIp_Header_ReturnCodeEnum::Unknown(0) }
}


/**
 * [PRS_SOMEIP_00035] The assignment of the Message ID shall be up to
 * the user. However, the Message ID shall be unique for the whole
 * system (i.e. the vehicle). TheMessage ID is similar to a CAN ID and
 * should be handled via a comparable process.
 * [PRS_SOMEIP_00038] Message IDs of method calls shall be structured in
 * the ID with 2^16 services with 2^15 methods.
 * \sa AUTOSAR_PRS_SOMEIPProtocol.pdf 4.1.1.1  Message ID
 */

#[derive(Default, Debug, Clone)]
pub struct SomeIp_Header_MessageId {
    pub(crate) _root: SharedType<SomeIp>,
    pub(crate) _parent: SharedType<SomeIp_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    service_id: RefCell<u16>,
    sub_id: RefCell<bool>,
    method_id: RefCell<u64>,
    event_id: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<u32>,
}
impl KStruct for SomeIp_Header_MessageId {
    type Root = SomeIp;
    type Parent = SomeIp_Header;

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
        *self_rc.service_id.borrow_mut() = _io.read_u2be()?;
        *self_rc.sub_id.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        if *self_rc.sub_id() == false {
            *self_rc.method_id.borrow_mut() = _io.read_bits_int_be(15)?;
        }
        if *self_rc.sub_id() == true {
            *self_rc.event_id.borrow_mut() = _io.read_bits_int_be(15)?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SomeIp_Header_MessageId {

    /**
     * The value provides the undissected Message ID
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.value.borrow_mut() = _io.read_u4be()?;
        _io.seek(_pos)?;
        Ok(self.value.borrow())
    }
}

/**
 * Service ID
 */
impl SomeIp_Header_MessageId {
    pub fn service_id(&self) -> Ref<'_, u16> {
        self.service_id.borrow()
    }
}

/**
 * Single bit to flag, if there is a Method or a Event ID
 */
impl SomeIp_Header_MessageId {
    pub fn sub_id(&self) -> Ref<'_, bool> {
        self.sub_id.borrow()
    }
}

/**
 * Method ID
 * \sa AUTOSAR_PRS_SOMEIPProtocol.pdf - Table 4.1.
 */
impl SomeIp_Header_MessageId {
    pub fn method_id(&self) -> Ref<'_, u64> {
        self.method_id.borrow()
    }
}

/**
 * Event ID
 * \sa AUTOSAR_PRS_SOMEIPProtocol.pdf - Table 4.6
 */
impl SomeIp_Header_MessageId {
    pub fn event_id(&self) -> Ref<'_, u64> {
        self.event_id.borrow()
    }
}
impl SomeIp_Header_MessageId {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * The Request ID allows a provider and subscriber to differentiate
 * multiple parallel usesof the same method, event, getter or setter.
 * \sa AUTOSAR_PRS_SOMEIPProtocol.pdf - section 4.1.1.3  Request ID
 */

#[derive(Default, Debug, Clone)]
pub struct SomeIp_Header_RequestId {
    pub(crate) _root: SharedType<SomeIp>,
    pub(crate) _parent: SharedType<SomeIp_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    client_id: RefCell<u16>,
    session_id: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<u32>,
}
impl KStruct for SomeIp_Header_RequestId {
    type Root = SomeIp;
    type Parent = SomeIp_Header;

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
        *self_rc.client_id.borrow_mut() = _io.read_u2be()?;
        *self_rc.session_id.borrow_mut() = _io.read_u2be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SomeIp_Header_RequestId {

    /**
     * The value provides the undissected Request ID
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.value.borrow_mut() = _io.read_u4be()?;
        _io.seek(_pos)?;
        Ok(self.value.borrow())
    }
}
impl SomeIp_Header_RequestId {
    pub fn client_id(&self) -> Ref<'_, u16> {
        self.client_id.borrow()
    }
}
impl SomeIp_Header_RequestId {
    pub fn session_id(&self) -> Ref<'_, u16> {
        self.session_id.borrow()
    }
}
impl SomeIp_Header_RequestId {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
