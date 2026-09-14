// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/*
== License information for parts derived from kaitai_struct_tests, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

MIT License

Copyright (c) 2019 Kaitai Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


*/
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * describes the first 4 header bytes of a TS Packet header
 */

#[derive(Default, Debug, Clone)]
pub struct TsPacketHeader {
    pub(crate) _root: SharedType<TsPacketHeader>,
    pub(crate) _parent: SharedType<TsPacketHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    sync_byte: RefCell<u8>,
    transport_error_indicator: RefCell<bool>,
    payload_unit_start_indicator: RefCell<bool>,
    transport_priority: RefCell<bool>,
    pid: RefCell<u64>,
    transport_scrambling_control: RefCell<u64>,
    adaptation_field_control: RefCell<TsPacketHeader_AdaptationFieldControlEnum>,
    continuity_counter: RefCell<u64>,
    ts_packet_remain: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    ts_packet_remain_raw: RefCell<Vec<u8>>,
}
impl KStruct for TsPacketHeader {
    type Root = TsPacketHeader;
    type Parent = TsPacketHeader;

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
        *self_rc.sync_byte.borrow_mut() = _io.read_u1()?;
        *self_rc.transport_error_indicator.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.payload_unit_start_indicator.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.transport_priority.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.pid.borrow_mut() = _io.read_bits_int_be(13)?;
        *self_rc.transport_scrambling_control.borrow_mut() = _io.read_bits_int_be(2)?;
        *self_rc.adaptation_field_control.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        *self_rc.continuity_counter.borrow_mut() = _io.read_bits_int_be(4)?;
        io.align_to_byte()?;
        *self_rc.ts_packet_remain.borrow_mut() = _io.read_bytes(184_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TsPacketHeader {
}
impl TsPacketHeader {
    pub fn sync_byte(&self) -> Ref<'_, u8> {
        self.sync_byte.borrow()
    }
}
impl TsPacketHeader {
    pub fn transport_error_indicator(&self) -> Ref<'_, bool> {
        self.transport_error_indicator.borrow()
    }
}
impl TsPacketHeader {
    pub fn payload_unit_start_indicator(&self) -> Ref<'_, bool> {
        self.payload_unit_start_indicator.borrow()
    }
}
impl TsPacketHeader {
    pub fn transport_priority(&self) -> Ref<'_, bool> {
        self.transport_priority.borrow()
    }
}
impl TsPacketHeader {
    pub fn pid(&self) -> Ref<'_, u64> {
        self.pid.borrow()
    }
}
impl TsPacketHeader {
    pub fn transport_scrambling_control(&self) -> Ref<'_, u64> {
        self.transport_scrambling_control.borrow()
    }
}
impl TsPacketHeader {
    pub fn adaptation_field_control(&self) -> Ref<'_, TsPacketHeader_AdaptationFieldControlEnum> {
        self.adaptation_field_control.borrow()
    }
}
impl TsPacketHeader {
    pub fn continuity_counter(&self) -> Ref<'_, u64> {
        self.continuity_counter.borrow()
    }
}
impl TsPacketHeader {
    pub fn ts_packet_remain(&self) -> Ref<'_, Vec<u8>> {
        self.ts_packet_remain.borrow()
    }
}
impl TsPacketHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl TsPacketHeader {
    pub fn ts_packet_remain_raw(&self) -> Ref<'_, Vec<u8>> {
        self.ts_packet_remain_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TsPacketHeader_AdaptationFieldControlEnum {
    Reserved,
    PayloadOnly,
    AdaptationFieldOnly,
    AdaptationFieldAndPayload,
    Unknown(i64),
}

impl TryFrom<i64> for TsPacketHeader_AdaptationFieldControlEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<TsPacketHeader_AdaptationFieldControlEnum> {
        match flag {
            0 => Ok(TsPacketHeader_AdaptationFieldControlEnum::Reserved),
            1 => Ok(TsPacketHeader_AdaptationFieldControlEnum::PayloadOnly),
            2 => Ok(TsPacketHeader_AdaptationFieldControlEnum::AdaptationFieldOnly),
            3 => Ok(TsPacketHeader_AdaptationFieldControlEnum::AdaptationFieldAndPayload),
            _ => Ok(TsPacketHeader_AdaptationFieldControlEnum::Unknown(flag)),
        }
    }
}

impl From<&TsPacketHeader_AdaptationFieldControlEnum> for i64 {
    fn from(v: &TsPacketHeader_AdaptationFieldControlEnum) -> Self {
        match *v {
            TsPacketHeader_AdaptationFieldControlEnum::Reserved => 0,
            TsPacketHeader_AdaptationFieldControlEnum::PayloadOnly => 1,
            TsPacketHeader_AdaptationFieldControlEnum::AdaptationFieldOnly => 2,
            TsPacketHeader_AdaptationFieldControlEnum::AdaptationFieldAndPayload => 3,
            TsPacketHeader_AdaptationFieldControlEnum::Unknown(v) => v
        }
    }
}

impl Default for TsPacketHeader_AdaptationFieldControlEnum {
    fn default() -> Self { TsPacketHeader_AdaptationFieldControlEnum::Unknown(0) }
}

