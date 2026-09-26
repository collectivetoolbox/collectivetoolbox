// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::protocol_body::ProtocolBody;

#[derive(Default, Debug, Clone)]
pub struct Ipv4Packet {
    pub(crate) _root: SharedType<Ipv4Packet>,
    pub(crate) _parent: SharedType<Ipv4Packet>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    b2: RefCell<u8>,
    total_length: RefCell<u16>,
    identification: RefCell<u16>,
    b67: RefCell<u16>,
    ttl: RefCell<u8>,
    protocol: RefCell<u8>,
    header_checksum: RefCell<u16>,
    src_ip_addr: RefCell<Vec<u8>>,
    dst_ip_addr: RefCell<Vec<u8>>,
    options: RefCell<OptRc<Ipv4Packet_Ipv4Options>>,
    body: RefCell<OptRc<ProtocolBody>>,
    _io: RefCell<BytesReader>,
    src_ip_addr_raw: RefCell<Vec<u8>>,
    dst_ip_addr_raw: RefCell<Vec<u8>>,
    options_raw: RefCell<Vec<u8>>,
    body_raw: RefCell<Vec<u8>>,
    f_ihl: Cell<bool>,
    ihl: RefCell<i32>,
    f_ihl_bytes: Cell<bool>,
    ihl_bytes: RefCell<i32>,
    f_version: Cell<bool>,
    version: RefCell<i32>,
}
impl KStruct for Ipv4Packet {
    type Root = Ipv4Packet;
    type Parent = Ipv4Packet;

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
        *self_rc.total_length.borrow_mut() = _io.read_u2be()?;
        *self_rc.identification.borrow_mut() = _io.read_u2be()?;
        *self_rc.b67.borrow_mut() = _io.read_u2be()?;
        *self_rc.ttl.borrow_mut() = _io.read_u1()?;
        *self_rc.protocol.borrow_mut() = _io.read_u1()?;
        *self_rc.header_checksum.borrow_mut() = _io.read_u2be()?;
        *self_rc.src_ip_addr.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc.dst_ip_addr.borrow_mut() = _io.read_bytes(4_usize)?;
        let _raw_options = _io.read_bytes(usize::try_from((*self_rc.ihl_bytes()?).saturating_sub(20_i32))?)?;
        *self_rc.options_raw.borrow_mut() = _raw_options.clone();
        let _io_options = BytesReader::from(_raw_options);
        let t = Self::read_into::<BytesReader, Ipv4Packet_Ipv4Options>(&_io_options, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.options.borrow_mut() = t;
        let _raw_body = _io.read_bytes(usize::try_from((i32::from(*self_rc.total_length())).saturating_sub(*self_rc.ihl_bytes()?))?)?;
        *self_rc.body_raw.borrow_mut() = _raw_body.clone();
        let _io_body = BytesReader::from(_raw_body);
        let f = |t : &mut ProtocolBody| Ok(t.set_params((*self_rc.protocol()).try_into().map_err(|_| KError::CastError)?));
        let t = Self::read_into_with_init::<BytesReader, ProtocolBody>(&_io_body, None, None, &f)?.into();
        *self_rc.body.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Ipv4Packet {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn ihl(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_ihl.get() {
            return Ok(self.ihl.borrow());
        }
        self.f_ihl.set(true);
        *self.ihl.borrow_mut() = (((i32::from(*self.b1())) & (15_i32))).try_into()?;
        Ok(self.ihl.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn ihl_bytes(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_ihl_bytes.get() {
            return Ok(self.ihl_bytes.borrow());
        }
        self.f_ihl_bytes.set(true);
        *self.ihl_bytes.borrow_mut() = ((*self.ihl()?).saturating_mul(4_i32)).try_into()?;
        Ok(self.ihl_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn version(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_version.get() {
            return Ok(self.version.borrow());
        }
        self.f_version.set(true);
        *self.version.borrow_mut() = ((((i32::from(*self.b1())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.version.borrow())
    }
}
impl Ipv4Packet {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl Ipv4Packet {
    pub fn b2(&self) -> Ref<'_, u8> {
        self.b2.borrow()
    }
}
impl Ipv4Packet {
    pub fn total_length(&self) -> Ref<'_, u16> {
        self.total_length.borrow()
    }
}
impl Ipv4Packet {
    pub fn identification(&self) -> Ref<'_, u16> {
        self.identification.borrow()
    }
}
impl Ipv4Packet {
    pub fn b67(&self) -> Ref<'_, u16> {
        self.b67.borrow()
    }
}
impl Ipv4Packet {
    pub fn ttl(&self) -> Ref<'_, u8> {
        self.ttl.borrow()
    }
}
impl Ipv4Packet {
    pub fn protocol(&self) -> Ref<'_, u8> {
        self.protocol.borrow()
    }
}
impl Ipv4Packet {
    pub fn header_checksum(&self) -> Ref<'_, u16> {
        self.header_checksum.borrow()
    }
}
impl Ipv4Packet {
    pub fn src_ip_addr(&self) -> Ref<'_, Vec<u8>> {
        self.src_ip_addr.borrow()
    }
}
impl Ipv4Packet {
    pub fn dst_ip_addr(&self) -> Ref<'_, Vec<u8>> {
        self.dst_ip_addr.borrow()
    }
}
impl Ipv4Packet {
    pub fn options(&self) -> Ref<'_, OptRc<Ipv4Packet_Ipv4Options>> {
        self.options.borrow()
    }
}
impl Ipv4Packet {
    pub fn body(&self) -> Ref<'_, OptRc<ProtocolBody>> {
        self.body.borrow()
    }
}
impl Ipv4Packet {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Ipv4Packet {
    pub fn src_ip_addr_raw(&self) -> Ref<'_, Vec<u8>> {
        self.src_ip_addr_raw.borrow()
    }
}
impl Ipv4Packet {
    pub fn dst_ip_addr_raw(&self) -> Ref<'_, Vec<u8>> {
        self.dst_ip_addr_raw.borrow()
    }
}
impl Ipv4Packet {
    pub fn options_raw(&self) -> Ref<'_, Vec<u8>> {
        self.options_raw.borrow()
    }
}
impl Ipv4Packet {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ipv4Packet_Ipv4Option {
    pub(crate) _root: SharedType<Ipv4Packet>,
    pub(crate) _parent: SharedType<Ipv4Packet_Ipv4Options>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    len: RefCell<u8>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
    f_copy: Cell<bool>,
    copy: RefCell<i32>,
    f_number: Cell<bool>,
    number: RefCell<i32>,
    f_opt_class: Cell<bool>,
    opt_class: RefCell<i32>,
}
impl KStruct for Ipv4Packet_Ipv4Option {
    type Root = Ipv4Packet;
    type Parent = Ipv4Packet_Ipv4Options;

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
        *self_rc.len.borrow_mut() = _io.read_u1()?;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(if ((to_i128(*self_rc.len())) > (to_i128(2))) { (i32::from(*self_rc.len())).saturating_sub(2_i32) } else { 0_i32 })?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Ipv4Packet_Ipv4Option {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn copy(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_copy.get() {
            return Ok(self.copy.borrow());
        }
        self.f_copy.set(true);
        *self.copy.borrow_mut() = ((((i32::from(*self.b1())) & (128_i32))).wrapping_shr(7_u32)).try_into()?;
        Ok(self.copy.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn number(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_number.get() {
            return Ok(self.number.borrow());
        }
        self.f_number.set(true);
        *self.number.borrow_mut() = (((i32::from(*self.b1())) & (31_i32))).try_into()?;
        Ok(self.number.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn opt_class(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_opt_class.get() {
            return Ok(self.opt_class.borrow());
        }
        self.f_opt_class.set(true);
        *self.opt_class.borrow_mut() = ((((i32::from(*self.b1())) & (96_i32))).wrapping_shr(5_u32)).try_into()?;
        Ok(self.opt_class.borrow())
    }
}
impl Ipv4Packet_Ipv4Option {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl Ipv4Packet_Ipv4Option {
    pub fn len(&self) -> Ref<'_, u8> {
        self.len.borrow()
    }
}
impl Ipv4Packet_Ipv4Option {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl Ipv4Packet_Ipv4Option {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Ipv4Packet_Ipv4Option {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ipv4Packet_Ipv4Options {
    pub(crate) _root: SharedType<Ipv4Packet>,
    pub(crate) _parent: SharedType<Ipv4Packet>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<Ipv4Packet_Ipv4Option>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ipv4Packet_Ipv4Options {
    type Root = Ipv4Packet;
    type Parent = Ipv4Packet;

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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Ipv4Packet_Ipv4Option>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Ipv4Packet_Ipv4Options {
}
impl Ipv4Packet_Ipv4Options {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<Ipv4Packet_Ipv4Option>>> {
        self.entries.borrow()
    }
}
impl Ipv4Packet_Ipv4Options {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
