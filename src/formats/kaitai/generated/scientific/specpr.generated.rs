// SPDX-License-Identifier: Unlicense
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Specpr records are fixed format, 1536 bytes/record. Record number
 * counting starts at 0. Binary data are in IEEE format real numbers
 * and non-byte swapped integers (compatiible with all Sun
 * Microsystems, and Hewlett Packard workstations (Intel and some DEC
 * machines are byte swapped relative to Suns and HPs). Each record may
 * contain different information according to the following scheme.
 *
 * You can get some library of spectra from
 * ftp://ftpext.cr.usgs.gov/pub/cr/co/denver/speclab/pub/spectral.library/splib06.library/
 */

#[derive(Default, Debug, Clone)]
pub struct Specpr {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr>,
    pub(crate) _self_shared: SharedType<Self>,
    records: RefCell<Vec<OptRc<Specpr_Record>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Specpr {
    type Root = Specpr;
    type Parent = Specpr;

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
        *self_rc.records.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Specpr_Record>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr {
}
impl Specpr {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<Specpr_Record>>> {
        self.records.borrow()
    }
}
impl Specpr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Specpr_RecordType {
    DataInitial,
    TextInitial,
    DataContinuation,
    TextContinuation,
    Unknown(i64),
}

impl TryFrom<i64> for Specpr_RecordType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Specpr_RecordType> {
        match flag {
            0 => Ok(Specpr_RecordType::DataInitial),
            1 => Ok(Specpr_RecordType::TextInitial),
            2 => Ok(Specpr_RecordType::DataContinuation),
            3 => Ok(Specpr_RecordType::TextContinuation),
            _ => Ok(Specpr_RecordType::Unknown(flag)),
        }
    }
}

impl From<&Specpr_RecordType> for i64 {
    fn from(v: &Specpr_RecordType) -> Self {
        match *v {
            Specpr_RecordType::DataInitial => 0,
            Specpr_RecordType::TextInitial => 1,
            Specpr_RecordType::DataContinuation => 2,
            Specpr_RecordType::TextContinuation => 3,
            Specpr_RecordType::Unknown(v) => v
        }
    }
}

impl Default for Specpr_RecordType {
    fn default() -> Self { Specpr_RecordType::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Specpr_CoarseTimestamp {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_DataInitial>,
    pub(crate) _self_shared: SharedType<Self>,
    scaled_seconds: RefCell<i32>,
    _io: RefCell<BytesReader>,
    f_seconds: Cell<bool>,
    seconds: RefCell<f64>,
}
impl KStruct for Specpr_CoarseTimestamp {
    type Root = Specpr;
    type Parent = Specpr_DataInitial;

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
        *self_rc.scaled_seconds.borrow_mut() = _io.read_s4be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_CoarseTimestamp {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seconds(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_seconds.get() {
            return Ok(self.seconds.borrow());
        }
        self.f_seconds.set(true);
        *self.seconds.borrow_mut() = (((to_f64(*self.scaled_seconds())) * (24000.0))).try_into()?;
        Ok(self.seconds.borrow())
    }
}
impl Specpr_CoarseTimestamp {
    pub fn scaled_seconds(&self) -> Ref<'_, i32> {
        self.scaled_seconds.borrow()
    }
}
impl Specpr_CoarseTimestamp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_DataContinuation {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    cdata: RefCell<Vec<f32>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Specpr_DataContinuation {
    type Root = Specpr;
    type Parent = Specpr_Record;

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
        *self_rc.cdata.borrow_mut() = Vec::new();
        let l_cdata = 383_usize;
        for _i in 0_usize..l_cdata {
            self_rc.cdata.borrow_mut().push(_io.read_f4be()?);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_DataContinuation {
}

/**
 * The continuation of the data values (383 channels of 32 bit real numbers).
 */
impl Specpr_DataContinuation {
    pub fn cdata(&self) -> Ref<'_, Vec<f32>> {
        self.cdata.borrow()
    }
}
impl Specpr_DataContinuation {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_DataInitial {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    ids: RefCell<OptRc<Specpr_Identifiers>>,
    iscta: RefCell<OptRc<Specpr_CoarseTimestamp>>,
    isctb: RefCell<OptRc<Specpr_CoarseTimestamp>>,
    jdatea: RefCell<i32>,
    jdateb: RefCell<i32>,
    istb: RefCell<OptRc<Specpr_CoarseTimestamp>>,
    isra: RefCell<i32>,
    isdec: RefCell<i32>,
    itchan: RefCell<i32>,
    irmas: RefCell<i32>,
    revs: RefCell<i32>,
    iband: RefCell<Vec<i32>>,
    irwav: RefCell<i32>,
    irespt: RefCell<i32>,
    irecno: RefCell<i32>,
    itpntr: RefCell<i32>,
    ihist: RefCell<String>,
    mhist: RefCell<Vec<String>>,
    nruns: RefCell<i32>,
    siangl: RefCell<OptRc<Specpr_IllumAngle>>,
    seangl: RefCell<OptRc<Specpr_IllumAngle>>,
    sphase: RefCell<i32>,
    iwtrns: RefCell<i32>,
    itimch: RefCell<i32>,
    xnrm: RefCell<f32>,
    scatim: RefCell<f32>,
    timint: RefCell<f32>,
    tempd: RefCell<f32>,
    data: RefCell<Vec<f32>>,
    _io: RefCell<BytesReader>,
    ihist_raw: RefCell<Vec<u8>>,
    mhist_raw: RefCell<Vec<u8>>,
    f_phase_angle_arcsec: Cell<bool>,
    phase_angle_arcsec: RefCell<f64>,
}
impl KStruct for Specpr_DataInitial {
    type Root = Specpr;
    type Parent = Specpr_Record;

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
        let t = Self::read_into::<_, Specpr_Identifiers>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.ids.borrow_mut() = t;
        let t = Self::read_into::<_, Specpr_CoarseTimestamp>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.iscta.borrow_mut() = t;
        let t = Self::read_into::<_, Specpr_CoarseTimestamp>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.isctb.borrow_mut() = t;
        *self_rc.jdatea.borrow_mut() = _io.read_s4be()?;
        *self_rc.jdateb.borrow_mut() = _io.read_s4be()?;
        let t = Self::read_into::<_, Specpr_CoarseTimestamp>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.istb.borrow_mut() = t;
        *self_rc.isra.borrow_mut() = _io.read_s4be()?;
        *self_rc.isdec.borrow_mut() = _io.read_s4be()?;
        *self_rc.itchan.borrow_mut() = _io.read_s4be()?;
        *self_rc.irmas.borrow_mut() = _io.read_s4be()?;
        *self_rc.revs.borrow_mut() = _io.read_s4be()?;
        *self_rc.iband.borrow_mut() = Vec::new();
        let l_iband = 2_usize;
        for _i in 0_usize..l_iband {
            self_rc.iband.borrow_mut().push(_io.read_s4be()?);
        }
        *self_rc.irwav.borrow_mut() = _io.read_s4be()?;
        *self_rc.irespt.borrow_mut() = _io.read_s4be()?;
        *self_rc.irecno.borrow_mut() = _io.read_s4be()?;
        *self_rc.itpntr.borrow_mut() = _io.read_s4be()?;
        *self_rc.ihist.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(60_usize)?, None, false, Some(32)), "ascii")?;
        *self_rc.mhist.borrow_mut() = Vec::new();
        let l_mhist = 4_usize;
        for _i in 0_usize..l_mhist {
            self_rc.mhist.borrow_mut().push(bytes_to_str(&_io.read_bytes(74_usize)?, "ascii")?);
        }
        *self_rc.nruns.borrow_mut() = _io.read_s4be()?;
        let t = Self::read_into::<_, Specpr_IllumAngle>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.siangl.borrow_mut() = t;
        let t = Self::read_into::<_, Specpr_IllumAngle>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.seangl.borrow_mut() = t;
        *self_rc.sphase.borrow_mut() = _io.read_s4be()?;
        *self_rc.iwtrns.borrow_mut() = _io.read_s4be()?;
        *self_rc.itimch.borrow_mut() = _io.read_s4be()?;
        *self_rc.xnrm.borrow_mut() = _io.read_f4be()?;
        *self_rc.scatim.borrow_mut() = _io.read_f4be()?;
        *self_rc.timint.borrow_mut() = _io.read_f4be()?;
        *self_rc.tempd.borrow_mut() = _io.read_f4be()?;
        *self_rc.data.borrow_mut() = Vec::new();
        let l_data = 256_usize;
        for _i in 0_usize..l_data {
            self_rc.data.borrow_mut().push(_io.read_f4be()?);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_DataInitial {

    /**
     * The phase angle between iangl and eangl in seconds
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn phase_angle_arcsec(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_phase_angle_arcsec.get() {
            return Ok(self.phase_angle_arcsec.borrow());
        }
        self.f_phase_angle_arcsec.set(true);
        *self.phase_angle_arcsec.borrow_mut() = (((to_f64(*self.sphase())) / (1500.0))).try_into()?;
        Ok(self.phase_angle_arcsec.borrow())
    }
}
impl Specpr_DataInitial {
    pub fn ids(&self) -> Ref<'_, OptRc<Specpr_Identifiers>> {
        self.ids.borrow()
    }
}

/**
 * Civil or Universal time when data was last processed
 */
impl Specpr_DataInitial {
    pub fn iscta(&self) -> Ref<'_, OptRc<Specpr_CoarseTimestamp>> {
        self.iscta.borrow()
    }
}

/**
 * Civil or Universal time at the start of the spectral run
 */
impl Specpr_DataInitial {
    pub fn isctb(&self) -> Ref<'_, OptRc<Specpr_CoarseTimestamp>> {
        self.isctb.borrow()
    }
}

/**
 * Date when data was last processed. Stored as integer*4 Julian Day number *10
 */
impl Specpr_DataInitial {
    pub fn jdatea(&self) -> Ref<'_, i32> {
        self.jdatea.borrow()
    }
}

/**
 * Date when the spectral run began. Stored as integer*4 Julian Day number *10
 */
impl Specpr_DataInitial {
    pub fn jdateb(&self) -> Ref<'_, i32> {
        self.jdateb.borrow()
    }
}

/**
 * Siderial time when the spectral run started. See flag #05.
 */
impl Specpr_DataInitial {
    pub fn istb(&self) -> Ref<'_, OptRc<Specpr_CoarseTimestamp>> {
        self.istb.borrow()
    }
}

/**
 * Right ascension coordinates of an astronomical  object, or longitude on a planetary surface (integer*4 numbers in seconds *1000) (RA in RA seconds, Longitude in arc-seconds) See flag #06.
 */
impl Specpr_DataInitial {
    pub fn isra(&self) -> Ref<'_, i32> {
        self.isra.borrow()
    }
}

/**
 * Declination coordinates of an astronomical object, or latitude on a planetary surface (integer*4 number in arc-seconds *1000). See flag #06.
 */
impl Specpr_DataInitial {
    pub fn isdec(&self) -> Ref<'_, i32> {
        self.isdec.borrow()
    }
}

/**
 * Total number of channels in the spectrum (integer*4 value from 1 to 4852)
 */
impl Specpr_DataInitial {
    pub fn itchan(&self) -> Ref<'_, i32> {
        self.itchan.borrow()
    }
}

/**
 * The equivalent atmospheric thickness through which the observation was obtained (=1.0 overhead scaled: airmass*1000; integer*4).
 */
impl Specpr_DataInitial {
    pub fn irmas(&self) -> Ref<'_, i32> {
        self.irmas.borrow()
    }
}

/**
 * The number of independent spectral scans which were added to make the spectrum (integer*4 number).
 */
impl Specpr_DataInitial {
    pub fn revs(&self) -> Ref<'_, i32> {
        self.revs.borrow()
    }
}

/**
 * The channel numbers which define the band normalization (scaling to unity). (integers*4).
 */
impl Specpr_DataInitial {
    pub fn iband(&self) -> Ref<'_, Vec<i32>> {
        self.iband.borrow()
    }
}

/**
 * The record number within the file where the wavelengths are found (integer*4).
 */
impl Specpr_DataInitial {
    pub fn irwav(&self) -> Ref<'_, i32> {
        self.irwav.borrow()
    }
}

/**
 * The record pointer to where the resolution can be found (or horizontal error bar) (integer*4).
 */
impl Specpr_DataInitial {
    pub fn irespt(&self) -> Ref<'_, i32> {
        self.irespt.borrow()
    }
}

/**
 * The record number within the file where the data is located (integer*4 number).
 */
impl Specpr_DataInitial {
    pub fn irecno(&self) -> Ref<'_, i32> {
        self.irecno.borrow()
    }
}

/**
 * Text data record pointer. This pointer points to a data record where additional text describing the data may be found.  (32 bit integer)
 */
impl Specpr_DataInitial {
    pub fn itpntr(&self) -> Ref<'_, i32> {
        self.itpntr.borrow()
    }
}

/**
 * The program automatic 60 character history.
 */
impl Specpr_DataInitial {
    pub fn ihist(&self) -> Ref<'_, String> {
        self.ihist.borrow()
    }
}

/**
 * Manual history. Program automatic for large history requirements.
 */
impl Specpr_DataInitial {
    pub fn mhist(&self) -> Ref<'_, Vec<String>> {
        self.mhist.borrow()
    }
}

/**
 * The number of independent spectral runs which were summed or averaged to make this spectrum (integer*4).
 */
impl Specpr_DataInitial {
    pub fn nruns(&self) -> Ref<'_, i32> {
        self.nruns.borrow()
    }
}

/**
 * The angle of incidence of illuminating radiation
 *       integrating sphere = 2000000000
 *       Geometric albedo   = 2000000001
 */
impl Specpr_DataInitial {
    pub fn siangl(&self) -> Ref<'_, OptRc<Specpr_IllumAngle>> {
        self.siangl.borrow()
    }
}

/**
 * The angle of emission of illuminating radiation
 *       integrating sphere = 2000000000
 *       Geometric albedo   = 2000000001
 */
impl Specpr_DataInitial {
    pub fn seangl(&self) -> Ref<'_, OptRc<Specpr_IllumAngle>> {
        self.seangl.borrow()
    }
}

/**
 * The phase angle between iangl and eangl (Integer*4 number, in arc-seconds*1500). (180 degrees=972000000; -180 deg <= phase <= 180 deg)
 *       integrating sphere = 2000000000
 */
impl Specpr_DataInitial {
    pub fn sphase(&self) -> Ref<'_, i32> {
        self.sphase.borrow()
    }
}

/**
 * Weighted number of runs (the number of runs of the spectrum with the minimum runs which was used in processing this spectrum, integer*4).
 */
impl Specpr_DataInitial {
    pub fn iwtrns(&self) -> Ref<'_, i32> {
        self.iwtrns.borrow()
    }
}

/**
 * The time observed in the sample beam for each half chop in milliseconds (for chopping spectrometers only). (integer*4)
 */
impl Specpr_DataInitial {
    pub fn itimch(&self) -> Ref<'_, i32> {
        self.itimch.borrow()
    }
}

/**
 * The band normalization factor. For data scaled to 1.0, multiply by this number to recover photometric level (32 bit real number).
 */
impl Specpr_DataInitial {
    pub fn xnrm(&self) -> Ref<'_, f32> {
        self.xnrm.borrow()
    }
}

/**
 * The time it takes to make one scan of the entire spectrum in seconds (32 bit real number).
 */
impl Specpr_DataInitial {
    pub fn scatim(&self) -> Ref<'_, f32> {
        self.scatim.borrow()
    }
}

/**
 * Total integration time (usually=scatime * nruns) (32 bit real number).
 */
impl Specpr_DataInitial {
    pub fn timint(&self) -> Ref<'_, f32> {
        self.timint.borrow()
    }
}

/**
 * Temperature in degrees Kelvin (32 bit real number).
 */
impl Specpr_DataInitial {
    pub fn tempd(&self) -> Ref<'_, f32> {
        self.tempd.borrow()
    }
}

/**
 * The spectral data (256 channels of 32 bit real data numbers).
 */
impl Specpr_DataInitial {
    pub fn data(&self) -> Ref<'_, Vec<f32>> {
        self.data.borrow()
    }
}
impl Specpr_DataInitial {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Specpr_DataInitial {
    pub fn ihist_raw(&self) -> Ref<'_, Vec<u8>> {
        self.ihist_raw.borrow()
    }
}
impl Specpr_DataInitial {
    pub fn mhist_raw(&self) -> Ref<'_, Vec<u8>> {
        self.mhist_raw.borrow()
    }
}

/**
 * it is big endian
 */

#[derive(Default, Debug, Clone)]
pub struct Specpr_Icflag {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved: RefCell<u64>,
    isctb_type: RefCell<bool>,
    iscta_type: RefCell<bool>,
    coordinate_mode: RefCell<bool>,
    errors: RefCell<bool>,
    text: RefCell<bool>,
    continuation: RefCell<bool>,
    _io: RefCell<BytesReader>,
    f_type: Cell<bool>,
    r#type: RefCell<Specpr_RecordType>,
}
impl KStruct for Specpr_Icflag {
    type Root = Specpr;
    type Parent = Specpr_Record;

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
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(26)?;
        *self_rc.isctb_type.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.iscta_type.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.coordinate_mode.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.errors.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.text.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.continuation.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_Icflag {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn r#type(
        &self
    ) -> KResult<Ref<'_, Specpr_RecordType>> {
        let _io = self._io.borrow();
        if self.f_type.get() {
            return Ok(self.r#type.borrow());
        }
        self.f_type.set(true);
        *self.r#type.borrow_mut() = i64::from((((if *self.text() { 1_i32 } else { 0_i32 })).saturating_mul(1_i32)).saturating_add(((if *self.continuation() { 1_i32 } else { 0_i32 })).saturating_mul(2_i32))).try_into()?;
        Ok(self.r#type.borrow())
    }
}
impl Specpr_Icflag {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}

/**
 * =0 ctb is civil time
 * =1 ctb is universal time
 */
impl Specpr_Icflag {
    pub fn isctb_type(&self) -> Ref<'_, bool> {
        self.isctb_type.borrow()
    }
}

/**
 * =0 cta is civil time
 * =1 cta is universal time
 */
impl Specpr_Icflag {
    pub fn iscta_type(&self) -> Ref<'_, bool> {
        self.iscta_type.borrow()
    }
}

/**
 * RA, Dec / Long., Lat flag
 * =0 the array "ira" and "idec" corresponds to the right ascension and declination of an astronomical object.
 * =1 the array "ira" and "idec" correspond to the longitude and latitude of a spot on a planetary surface.
 */
impl Specpr_Icflag {
    pub fn coordinate_mode(&self) -> Ref<'_, bool> {
        self.coordinate_mode.borrow()
    }
}

/**
 * flag to indicate whether or not the data for the error bar (1 sigma standard deviation of the mean) is in the next record set. =0: no errors, =1: errors in next record set.
 */
impl Specpr_Icflag {
    pub fn errors(&self) -> Ref<'_, bool> {
        self.errors.borrow()
    }
}

/**
 * =0 the data in the array "data" is data
 * =1 the data in the array "data" is ascii text as is most of the header info.
 */
impl Specpr_Icflag {
    pub fn text(&self) -> Ref<'_, bool> {
        self.text.borrow()
    }
}

/**
 * =0 first record of a spectrum consists of: header then 256 data channels
 * =1 continuation data record consisting of:
 *   # bit flags followed by 1532 bytes of
 *   # real data (bit 1=0) (383 channels)
 *   # or 1532 bytes of text (bit 1=1).
 *   # A maximum of 12 continuation records
 *   # are allowed for a total of 4852
 *   # channels (limited by arrays of 4864)
 *   # or 19860 characters of text (bit 1=1).
 */
impl Specpr_Icflag {
    pub fn continuation(&self) -> Ref<'_, bool> {
        self.continuation.borrow()
    }
}
impl Specpr_Icflag {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_Identifiers {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    ititle: RefCell<String>,
    usernm: RefCell<String>,
    _io: RefCell<BytesReader>,
    ititle_raw: RefCell<Vec<u8>>,
    usernm_raw: RefCell<Vec<u8>>,
}
impl KStruct for Specpr_Identifiers {
    type Root = Specpr;
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
        *self_rc.ititle.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(40_usize)?, None, false, Some(32)), "ascii")?;
        *self_rc.usernm.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ascii")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_Identifiers {
}

/**
 * Title which describes the data
 */
impl Specpr_Identifiers {
    pub fn ititle(&self) -> Ref<'_, String> {
        self.ititle.borrow()
    }
}

/**
 * The name of the user who created the data record
 */
impl Specpr_Identifiers {
    pub fn usernm(&self) -> Ref<'_, String> {
        self.usernm.borrow()
    }
}
impl Specpr_Identifiers {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Specpr_Identifiers {
    pub fn ititle_raw(&self) -> Ref<'_, Vec<u8>> {
        self.ititle_raw.borrow()
    }
}
impl Specpr_Identifiers {
    pub fn usernm_raw(&self) -> Ref<'_, Vec<u8>> {
        self.usernm_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_IllumAngle {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_DataInitial>,
    pub(crate) _self_shared: SharedType<Self>,
    angl: RefCell<i32>,
    _io: RefCell<BytesReader>,
    f_degrees_total: Cell<bool>,
    degrees_total: RefCell<i32>,
    f_minutes_total: Cell<bool>,
    minutes_total: RefCell<i32>,
    f_seconds_total: Cell<bool>,
    seconds_total: RefCell<i32>,
}
impl KStruct for Specpr_IllumAngle {
    type Root = Specpr;
    type Parent = Specpr_DataInitial;

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
        *self_rc.angl.borrow_mut() = _io.read_s4be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_IllumAngle {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn degrees_total(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_degrees_total.get() {
            return Ok(self.degrees_total.borrow());
        }
        self.f_degrees_total.set(true);
        *self.degrees_total.borrow_mut() = (div_floor(i64::from(*self.minutes_total()?), 60_i64)?).try_into()?;
        Ok(self.degrees_total.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn minutes_total(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_minutes_total.get() {
            return Ok(self.minutes_total.borrow());
        }
        self.f_minutes_total.set(true);
        *self.minutes_total.borrow_mut() = (div_floor(i64::from(*self.seconds_total()?), 60_i64)?).try_into()?;
        Ok(self.minutes_total.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seconds_total(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_seconds_total.get() {
            return Ok(self.seconds_total.borrow());
        }
        self.f_seconds_total.set(true);
        *self.seconds_total.borrow_mut() = (div_floor(i64::from(*self.angl()), 6000_i64)?).try_into()?;
        Ok(self.seconds_total.borrow())
    }
}

/**
 * (Integer*4 number, in arc-seconds*6000). (90 degrees=1944000000; -90 deg <= angle <= 90 deg)
 */
impl Specpr_IllumAngle {
    pub fn angl(&self) -> Ref<'_, i32> {
        self.angl.borrow()
    }
}
impl Specpr_IllumAngle {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_Record {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr>,
    pub(crate) _self_shared: SharedType<Self>,
    icflag: RefCell<OptRc<Specpr_Icflag>>,
    content: RefCell<Option<Specpr_Record_Content>>,
    _io: RefCell<BytesReader>,
    content_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Specpr_Record_Content {
    Specpr_DataContinuation(OptRc<Specpr_DataContinuation>),
    Specpr_DataInitial(OptRc<Specpr_DataInitial>),
    Specpr_TextContinuation(OptRc<Specpr_TextContinuation>),
    Specpr_TextInitial(OptRc<Specpr_TextInitial>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Specpr_Record_Content> for OptRc<Specpr_DataContinuation> {
    type Error = KError;
    fn try_from(v: &Specpr_Record_Content) -> Result<Self, Self::Error> {
        if let Specpr_Record_Content::Specpr_DataContinuation(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Specpr_DataContinuation>> for Specpr_Record_Content {
    fn from(v: OptRc<Specpr_DataContinuation>) -> Self {
        Self::Specpr_DataContinuation(v)
    }
}
impl TryFrom<&Specpr_Record_Content> for OptRc<Specpr_DataInitial> {
    type Error = KError;
    fn try_from(v: &Specpr_Record_Content) -> Result<Self, Self::Error> {
        if let Specpr_Record_Content::Specpr_DataInitial(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Specpr_DataInitial>> for Specpr_Record_Content {
    fn from(v: OptRc<Specpr_DataInitial>) -> Self {
        Self::Specpr_DataInitial(v)
    }
}
impl TryFrom<&Specpr_Record_Content> for OptRc<Specpr_TextContinuation> {
    type Error = KError;
    fn try_from(v: &Specpr_Record_Content) -> Result<Self, Self::Error> {
        if let Specpr_Record_Content::Specpr_TextContinuation(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Specpr_TextContinuation>> for Specpr_Record_Content {
    fn from(v: OptRc<Specpr_TextContinuation>) -> Self {
        Self::Specpr_TextContinuation(v)
    }
}
impl TryFrom<&Specpr_Record_Content> for OptRc<Specpr_TextInitial> {
    type Error = KError;
    fn try_from(v: &Specpr_Record_Content) -> Result<Self, Self::Error> {
        if let Specpr_Record_Content::Specpr_TextInitial(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Specpr_TextInitial>> for Specpr_Record_Content {
    fn from(v: OptRc<Specpr_TextInitial>) -> Self {
        Self::Specpr_TextInitial(v)
    }
}
impl TryFrom<&Specpr_Record_Content> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Specpr_Record_Content) -> Result<Self, Self::Error> {
        if let Specpr_Record_Content::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Specpr_Record_Content {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Specpr_Record {
    type Root = Specpr;
    type Parent = Specpr;

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
        let t = Self::read_into::<_, Specpr_Icflag>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.icflag.borrow_mut() = t;
        match *self_rc.icflag().r#type()? {
            Specpr_RecordType::DataContinuation => {
                *self_rc.content_raw.borrow_mut() = _io.read_bytes(usize::try_from((1536_i32).saturating_sub(4_i32))?)?.into();
                let content_raw = self_rc.content_raw.borrow();
                let _t_content_raw_io = BytesReader::from(content_raw.clone());
                let t = Self::read_into::<BytesReader, Specpr_DataContinuation>(&_t_content_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Specpr_RecordType::DataInitial => {
                *self_rc.content_raw.borrow_mut() = _io.read_bytes(usize::try_from((1536_i32).saturating_sub(4_i32))?)?.into();
                let content_raw = self_rc.content_raw.borrow();
                let _t_content_raw_io = BytesReader::from(content_raw.clone());
                let t = Self::read_into::<BytesReader, Specpr_DataInitial>(&_t_content_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Specpr_RecordType::TextContinuation => {
                *self_rc.content_raw.borrow_mut() = _io.read_bytes(usize::try_from((1536_i32).saturating_sub(4_i32))?)?.into();
                let content_raw = self_rc.content_raw.borrow();
                let _t_content_raw_io = BytesReader::from(content_raw.clone());
                let t = Self::read_into::<BytesReader, Specpr_TextContinuation>(&_t_content_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.content.borrow_mut() = Some(t);
            }
            Specpr_RecordType::TextInitial => {
                *self_rc.content_raw.borrow_mut() = _io.read_bytes(usize::try_from((1536_i32).saturating_sub(4_i32))?)?.into();
                let content_raw = self_rc.content_raw.borrow();
                let _t_content_raw_io = BytesReader::from(content_raw.clone());
                let t = Self::read_into::<BytesReader, Specpr_TextInitial>(&_t_content_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
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
impl Specpr_Record {
}

/**
 * Total number of bytes comprising the document.
 */
impl Specpr_Record {
    pub fn icflag(&self) -> Ref<'_, OptRc<Specpr_Icflag>> {
        self.icflag.borrow()
    }
}
impl Specpr_Record {
    pub fn content(&self) -> Ref<'_, Option<Specpr_Record_Content>> {
        self.content.borrow()
    }
}
impl Specpr_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Specpr_Record {
    pub fn content_raw(&self) -> Ref<'_, Vec<u8>> {
        self.content_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_TextContinuation {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    tdata: RefCell<String>,
    _io: RefCell<BytesReader>,
    tdata_raw: RefCell<Vec<u8>>,
}
impl KStruct for Specpr_TextContinuation {
    type Root = Specpr;
    type Parent = Specpr_Record;

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
        *self_rc.tdata.borrow_mut() = bytes_to_str(&_io.read_bytes(1532_usize)?, "ascii")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_TextContinuation {
}

/**
 * 1532 characters of text.
 */
impl Specpr_TextContinuation {
    pub fn tdata(&self) -> Ref<'_, String> {
        self.tdata.borrow()
    }
}
impl Specpr_TextContinuation {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Specpr_TextContinuation {
    pub fn tdata_raw(&self) -> Ref<'_, Vec<u8>> {
        self.tdata_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Specpr_TextInitial {
    pub(crate) _root: SharedType<Specpr>,
    pub(crate) _parent: SharedType<Specpr_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    ids: RefCell<OptRc<Specpr_Identifiers>>,
    itxtpt: RefCell<u32>,
    itxtch: RefCell<i32>,
    itext: RefCell<String>,
    _io: RefCell<BytesReader>,
    itext_raw: RefCell<Vec<u8>>,
}
impl KStruct for Specpr_TextInitial {
    type Root = Specpr;
    type Parent = Specpr_Record;

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
        let t = Self::read_into::<_, Specpr_Identifiers>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.ids.borrow_mut() = t;
        *self_rc.itxtpt.borrow_mut() = _io.read_u4be()?;
        *self_rc.itxtch.borrow_mut() = _io.read_s4be()?;
        *self_rc.itext.borrow_mut() = bytes_to_str(&_io.read_bytes(1476_usize)?, "ascii")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Specpr_TextInitial {
}
impl Specpr_TextInitial {
    pub fn ids(&self) -> Ref<'_, OptRc<Specpr_Identifiers>> {
        self.ids.borrow()
    }
}

/**
 * Text data record pointer. This pointer points  to a data record where additional text may be may be found.
 */
impl Specpr_TextInitial {
    pub fn itxtpt(&self) -> Ref<'_, u32> {
        self.itxtpt.borrow()
    }
}

/**
 * The number of text characters (maximum= 19860).
 */
impl Specpr_TextInitial {
    pub fn itxtch(&self) -> Ref<'_, i32> {
        self.itxtch.borrow()
    }
}

/**
 * 1476 characters of text.  Text has embedded newlines so the number of lines available is limited only by the number of characters available.
 */
impl Specpr_TextInitial {
    pub fn itext(&self) -> Ref<'_, String> {
        self.itext.borrow()
    }
}
impl Specpr_TextInitial {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Specpr_TextInitial {
    pub fn itext_raw(&self) -> Ref<'_, Vec<u8>> {
        self.itext_raw.borrow()
    }
}
