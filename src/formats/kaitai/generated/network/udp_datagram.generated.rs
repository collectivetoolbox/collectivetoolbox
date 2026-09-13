// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * UDP is a simple stateless transport layer (AKA OSI layer 4)
 * protocol, one of the core Internet protocols. It provides source and
 * destination ports, basic checksumming, but provides not guarantees
 * of delivery, order of packets, or duplicate delivery.
 */

#[derive(Default, Debug, Clone)]
pub struct UdpDatagram {
    pub(crate) _root: SharedType<UdpDatagram>,
    pub(crate) _parent: SharedType<UdpDatagram>,
    pub(crate) _self_shared: SharedType<Self>,
    src_port: RefCell<u16>,
    dst_port: RefCell<u16>,
    length: RefCell<u16>,
    checksum: RefCell<u16>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for UdpDatagram {
    type Root = UdpDatagram;
    type Parent = UdpDatagram;

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
        *self_rc.src_port.borrow_mut() = _io.read_u2be()?;
        *self_rc.dst_port.borrow_mut() = _io.read_u2be()?;
        *self_rc.length.borrow_mut() = _io.read_u2be()?;
        *self_rc.checksum.borrow_mut() = _io.read_u2be()?;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from((i32::from(*self_rc.length())).saturating_sub(8_i32))?)?;
        Ok(())
    }
}
impl UdpDatagram {
}
impl UdpDatagram {
    pub fn src_port(&self) -> Ref<'_, u16> {
        self.src_port.borrow()
    }
}
impl UdpDatagram {
    pub fn dst_port(&self) -> Ref<'_, u16> {
        self.dst_port.borrow()
    }
}
impl UdpDatagram {
    pub fn length(&self) -> Ref<'_, u16> {
        self.length.borrow()
    }
}
impl UdpDatagram {
    pub fn checksum(&self) -> Ref<'_, u16> {
        self.checksum.borrow()
    }
}
impl UdpDatagram {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl UdpDatagram {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
