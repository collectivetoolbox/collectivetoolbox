// SPDX-License-Identifier: Unlicense
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Native format of Hashcat password "recovery" utility.
 *
 * A sample of file for testing can be downloaded from
 * <https://web.archive.org/web/20150220013635if_/http://hashcat.net:80/misc/example_hashes/hashcat.hccap>
 * \sa <https://hashcat.net/wiki/doku.php?id=hccap> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Hccap {
    pub(crate) _root: SharedType<Hccap>,
    pub(crate) _parent: SharedType<Hccap>,
    pub(crate) _self_shared: SharedType<Self>,
    records: RefCell<Vec<OptRc<Hccap_HccapRecord>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Hccap {
    type Root = Hccap;
    type Parent = Hccap;

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
        *self_rc.records.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Hccap_HccapRecord>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Hccap {
}
impl Hccap {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<Hccap_HccapRecord>>> {
        self.records.borrow()
    }
}
impl Hccap {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Hccap_EapolDummy {
    pub(crate) _root: SharedType<Hccap>,
    pub(crate) _parent: SharedType<Hccap_HccapRecord>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Hccap_EapolDummy {
    type Root = Hccap;
    type Parent = Hccap_HccapRecord;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Hccap_EapolDummy {
}
impl Hccap_EapolDummy {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Hccap_HccapRecord {
    pub(crate) _root: SharedType<Hccap>,
    pub(crate) _parent: SharedType<Hccap>,
    pub(crate) _self_shared: SharedType<Self>,
    essid: RefCell<Vec<u8>>,
    mac_ap: RefCell<Vec<u8>>,
    mac_station: RefCell<Vec<u8>>,
    nonce_station: RefCell<Vec<u8>>,
    nonce_ap: RefCell<Vec<u8>>,
    eapol_buffer: RefCell<OptRc<Hccap_EapolDummy>>,
    len_eapol: RefCell<u32>,
    keyver: RefCell<u32>,
    keymic: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    essid_raw: RefCell<Vec<u8>>,
    mac_ap_raw: RefCell<Vec<u8>>,
    mac_station_raw: RefCell<Vec<u8>>,
    nonce_station_raw: RefCell<Vec<u8>>,
    nonce_ap_raw: RefCell<Vec<u8>>,
    eapol_buffer_raw: RefCell<Vec<u8>>,
    keymic_raw: RefCell<Vec<u8>>,
    f_eapol: Cell<bool>,
    eapol: RefCell<Vec<u8>>,
}
impl KStruct for Hccap_HccapRecord {
    type Root = Hccap;
    type Parent = Hccap;

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
        *self_rc.essid.borrow_mut() = _io.read_bytes(36_usize)?;
        *self_rc.mac_ap.borrow_mut() = _io.read_bytes(6_usize)?;
        *self_rc.mac_station.borrow_mut() = _io.read_bytes(6_usize)?;
        *self_rc.nonce_station.borrow_mut() = _io.read_bytes(32_usize)?;
        *self_rc.nonce_ap.borrow_mut() = _io.read_bytes(32_usize)?;
        let _raw_eapol_buffer = _io.read_bytes(256_usize)?;
        *self_rc.eapol_buffer_raw.borrow_mut() = _raw_eapol_buffer.clone();
        let _io_eapol_buffer = BytesReader::from(_raw_eapol_buffer);
        let t = Self::read_into::<BytesReader, Hccap_EapolDummy>(&_io_eapol_buffer, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.eapol_buffer.borrow_mut() = t;
        *self_rc.len_eapol.borrow_mut() = _io.read_u4le()?;
        *self_rc.keyver.borrow_mut() = _io.read_u4le()?;
        *self_rc.keymic.borrow_mut() = _io.read_bytes(16_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Hccap_HccapRecord {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn eapol(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_eapol.get() {
            return Ok(self.eapol.borrow());
        }
        self.f_eapol.set(true);
        let io = KStream::clone(&*self.eapol_buffer()._io());
        let _pos = io.pos();
        io.seek(0_usize)?;
        *self.eapol.borrow_mut() = io.read_bytes(usize::try_from(*self.len_eapol())?)?;
        io.seek(_pos)?;
        Ok(self.eapol.borrow())
    }
}
impl Hccap_HccapRecord {
    pub fn essid(&self) -> Ref<'_, Vec<u8>> {
        self.essid.borrow()
    }
}

/**
 * The BSSID (MAC address) of the access point
 */
impl Hccap_HccapRecord {
    pub fn mac_ap(&self) -> Ref<'_, Vec<u8>> {
        self.mac_ap.borrow()
    }
}

/**
 * The MAC address of a client connecting to the access point
 */
impl Hccap_HccapRecord {
    pub fn mac_station(&self) -> Ref<'_, Vec<u8>> {
        self.mac_station.borrow()
    }
}

/**
 * Nonce (random salt) generated by the client connecting to the access point.
 */
impl Hccap_HccapRecord {
    pub fn nonce_station(&self) -> Ref<'_, Vec<u8>> {
        self.nonce_station.borrow()
    }
}

/**
 * Nonce (random salt) generated by the access point.
 */
impl Hccap_HccapRecord {
    pub fn nonce_ap(&self) -> Ref<'_, Vec<u8>> {
        self.nonce_ap.borrow()
    }
}

/**
 * Buffer for EAPOL data, only first `len_eapol` bytes are used
 */
impl Hccap_HccapRecord {
    pub fn eapol_buffer(&self) -> Ref<'_, OptRc<Hccap_EapolDummy>> {
        self.eapol_buffer.borrow()
    }
}

/**
 * Size of EAPOL data
 */
impl Hccap_HccapRecord {
    pub fn len_eapol(&self) -> Ref<'_, u32> {
        self.len_eapol.borrow()
    }
}

/**
 * The flag used to distinguish WPA from WPA2 ciphers. Value of
 * 1 means WPA, other - WPA2.
 */
impl Hccap_HccapRecord {
    pub fn keyver(&self) -> Ref<'_, u32> {
        self.keyver.borrow()
    }
}

/**
 * The final hash value. MD5 for WPA and SHA-1 for WPA2
 * (truncated to 128 bit).
 */
impl Hccap_HccapRecord {
    pub fn keymic(&self) -> Ref<'_, Vec<u8>> {
        self.keymic.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn essid_raw(&self) -> Ref<'_, Vec<u8>> {
        self.essid_raw.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn mac_ap_raw(&self) -> Ref<'_, Vec<u8>> {
        self.mac_ap_raw.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn mac_station_raw(&self) -> Ref<'_, Vec<u8>> {
        self.mac_station_raw.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn nonce_station_raw(&self) -> Ref<'_, Vec<u8>> {
        self.nonce_station_raw.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn nonce_ap_raw(&self) -> Ref<'_, Vec<u8>> {
        self.nonce_ap_raw.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn eapol_buffer_raw(&self) -> Ref<'_, Vec<u8>> {
        self.eapol_buffer_raw.borrow()
    }
}
impl Hccap_HccapRecord {
    pub fn keymic_raw(&self) -> Ref<'_, Vec<u8>> {
        self.keymic_raw.borrow()
    }
}
