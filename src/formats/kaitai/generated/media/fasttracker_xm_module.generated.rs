// SPDX-License-Identifier: Unlicense
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * XM (standing for eXtended Module) is a popular module music file
 * format, that was introduced in 1994 in FastTracker2 by Triton demo
 * group. Akin to MOD files, it bundles both digital samples
 * (instruments) and instructions on which note to play at what time
 * (patterns), which provides good audio quality with relatively small
 * file size. Audio is reproducible without relying on the sound of
 * particular hardware samplers or synths.
 * \sa <http://sid.ethz.ch/debian/milkytracker/milkytracker-0.90.85%2Bdfsg/resources/reference/xm-form.txt
ftp://ftp.modland.com/pub/documents/format_documentation/FastTracker%202%20v2.04%20(.xm).html
> Source
 */

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule>,
    pub(crate) _self_shared: SharedType<Self>,
    preheader: RefCell<OptRc<FasttrackerXmModule_Preheader>>,
    header: RefCell<OptRc<FasttrackerXmModule_Header>>,
    patterns: RefCell<Vec<OptRc<FasttrackerXmModule_Pattern>>>,
    instruments: RefCell<Vec<OptRc<FasttrackerXmModule_Instrument>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule;

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
        let t = Self::read_into::<_, FasttrackerXmModule_Preheader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.preheader.borrow_mut() = t;
        let t = Self::read_into::<_, FasttrackerXmModule_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.patterns.borrow_mut() = Vec::new();
        let l_patterns = *self_rc.header().num_patterns();
        for _i in 0..l_patterns {
            let t = Self::read_into::<_, FasttrackerXmModule_Pattern>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.patterns.borrow_mut().push(t);
        }
        *self_rc.instruments.borrow_mut() = Vec::new();
        let l_instruments = *self_rc.header().num_instruments();
        for _i in 0..l_instruments {
            let t = Self::read_into::<_, FasttrackerXmModule_Instrument>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.instruments.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl FasttrackerXmModule {
}
impl FasttrackerXmModule {
    pub fn preheader(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Preheader>> {
        self.preheader.borrow()
    }
}
impl FasttrackerXmModule {
    pub fn header(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Header>> {
        self.header.borrow()
    }
}
impl FasttrackerXmModule {
    pub fn patterns(&self) -> Ref<'_, Vec<OptRc<FasttrackerXmModule_Pattern>>> {
        self.patterns.borrow()
    }
}
impl FasttrackerXmModule {
    pub fn instruments(&self) -> Ref<'_, Vec<OptRc<FasttrackerXmModule_Instrument>>> {
        self.instruments.borrow()
    }
}
impl FasttrackerXmModule {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Flags {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved: RefCell<u64>,
    freq_table_type: RefCell<bool>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Flags {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Header;

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
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(15)?;
        *self_rc.freq_table_type.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        Ok(())
    }
}
impl FasttrackerXmModule_Flags {
}
impl FasttrackerXmModule_Flags {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}

/**
 * 0 = Amiga frequency table (see below); 1 = Linear frequency table
 */
impl FasttrackerXmModule_Flags {
    pub fn freq_table_type(&self) -> Ref<'_, bool> {
        self.freq_table_type.borrow()
    }
}
impl FasttrackerXmModule_Flags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Header {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule>,
    pub(crate) _self_shared: SharedType<Self>,
    song_length: RefCell<u16>,
    restart_position: RefCell<u16>,
    num_channels: RefCell<u16>,
    num_patterns: RefCell<u16>,
    num_instruments: RefCell<u16>,
    flags: RefCell<OptRc<FasttrackerXmModule_Flags>>,
    default_tempo: RefCell<u16>,
    default_bpm: RefCell<u16>,
    pattern_order_table: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Header {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule;

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
        *self_rc.song_length.borrow_mut() = _io.read_u2le()?;
        *self_rc.restart_position.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_channels.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_patterns.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_instruments.borrow_mut() = _io.read_u2le()?;
        let t = Self::read_into::<_, FasttrackerXmModule_Flags>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags.borrow_mut() = t;
        *self_rc.default_tempo.borrow_mut() = _io.read_u2le()?;
        *self_rc.default_bpm.borrow_mut() = _io.read_u2le()?;
        *self_rc.pattern_order_table.borrow_mut() = Vec::new();
        let l_pattern_order_table = 256;
        for _i in 0..l_pattern_order_table {
            self_rc.pattern_order_table.borrow_mut().push(_io.read_u1()?);
        }
        Ok(())
    }
}
impl FasttrackerXmModule_Header {
}

/**
 * Song length (in pattern order table)
 */
impl FasttrackerXmModule_Header {
    pub fn song_length(&self) -> Ref<'_, u16> {
        self.song_length.borrow()
    }
}
impl FasttrackerXmModule_Header {
    pub fn restart_position(&self) -> Ref<'_, u16> {
        self.restart_position.borrow()
    }
}

/**
 * (2,4,6,8,10,...,32)
 */
impl FasttrackerXmModule_Header {
    pub fn num_channels(&self) -> Ref<'_, u16> {
        self.num_channels.borrow()
    }
}

/**
 * (max 256)
 */
impl FasttrackerXmModule_Header {
    pub fn num_patterns(&self) -> Ref<'_, u16> {
        self.num_patterns.borrow()
    }
}

/**
 * (max 128)
 */
impl FasttrackerXmModule_Header {
    pub fn num_instruments(&self) -> Ref<'_, u16> {
        self.num_instruments.borrow()
    }
}
impl FasttrackerXmModule_Header {
    pub fn flags(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Flags>> {
        self.flags.borrow()
    }
}
impl FasttrackerXmModule_Header {
    pub fn default_tempo(&self) -> Ref<'_, u16> {
        self.default_tempo.borrow()
    }
}
impl FasttrackerXmModule_Header {
    pub fn default_bpm(&self) -> Ref<'_, u16> {
        self.default_bpm.borrow()
    }
}

/**
 * max 256
 */
impl FasttrackerXmModule_Header {
    pub fn pattern_order_table(&self) -> Ref<'_, Vec<u8>> {
        self.pattern_order_table.borrow()
    }
}
impl FasttrackerXmModule_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * XM's notion of "instrument" typically constitutes of a
 * instrument metadata and one or several samples. Metadata
 * includes:
 *
 * * instrument's name
 * * instruction of which sample to use for which note
 * * volume and panning envelopes and looping instructions
 * * vibrato settings
 */

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule>,
    pub(crate) _self_shared: SharedType<Self>,
    header_size: RefCell<u32>,
    header: RefCell<OptRc<FasttrackerXmModule_Instrument_Header>>,
    samples_headers: RefCell<Vec<OptRc<FasttrackerXmModule_Instrument_SampleHeader>>>,
    samples: RefCell<Vec<OptRc<FasttrackerXmModule_Instrument_SamplesData>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule;

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
        *self_rc.header_size.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, FasttrackerXmModule_Instrument_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.samples_headers.borrow_mut() = Vec::new();
        let l_samples_headers = *self_rc.header().num_samples();
        for _i in 0..l_samples_headers {
            let t = Self::read_into::<_, FasttrackerXmModule_Instrument_SampleHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.samples_headers.borrow_mut().push(t);
        }
        *self_rc.samples.borrow_mut() = Vec::new();
        let l_samples = *self_rc.header().num_samples();
        for _i in 0..l_samples {
            let f = |t : &mut FasttrackerXmModule_Instrument_SamplesData| Ok(t.set_params(self_rc.samples_headers().get(usize::try_from(_i)?).ok_or(KError::CastError)?.clone()));
            let t = Self::read_into_with_init::<_, FasttrackerXmModule_Instrument_SamplesData>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
            self_rc.samples.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument {
}

/**
 * Instrument size << header that is >>
 * << "Instrument Size" field tends to be more than the actual size of the structure documented here (it includes also the extended instrument sample header above). So remember to check it and skip the additional bytes before the first sample header >>
 */
impl FasttrackerXmModule_Instrument {
    pub fn header_size(&self) -> Ref<'_, u32> {
        self.header_size.borrow()
    }
}
impl FasttrackerXmModule_Instrument {
    pub fn header(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Instrument_Header>> {
        self.header.borrow()
    }
}
impl FasttrackerXmModule_Instrument {
    pub fn samples_headers(&self) -> Ref<'_, Vec<OptRc<FasttrackerXmModule_Instrument_SampleHeader>>> {
        self.samples_headers.borrow()
    }
}
impl FasttrackerXmModule_Instrument {
    pub fn samples(&self) -> Ref<'_, Vec<OptRc<FasttrackerXmModule_Instrument_SamplesData>>> {
        self.samples.borrow()
    }
}
impl FasttrackerXmModule_Instrument {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument_ExtraHeader {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Instrument_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    len_sample_header: RefCell<u32>,
    idx_sample_per_note: RefCell<Vec<u8>>,
    volume_points: RefCell<Vec<OptRc<FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint>>>,
    panning_points: RefCell<Vec<OptRc<FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint>>>,
    num_volume_points: RefCell<u8>,
    num_panning_points: RefCell<u8>,
    volume_sustain_point: RefCell<u8>,
    volume_loop_start_point: RefCell<u8>,
    volume_loop_end_point: RefCell<u8>,
    panning_sustain_point: RefCell<u8>,
    panning_loop_start_point: RefCell<u8>,
    panning_loop_end_point: RefCell<u8>,
    volume_type: RefCell<FasttrackerXmModule_Instrument_ExtraHeader_Type>,
    panning_type: RefCell<FasttrackerXmModule_Instrument_ExtraHeader_Type>,
    vibrato_type: RefCell<u8>,
    vibrato_sweep: RefCell<u8>,
    vibrato_depth: RefCell<u8>,
    vibrato_rate: RefCell<u8>,
    volume_fadeout: RefCell<u16>,
    reserved: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument_ExtraHeader {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Instrument_Header;

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
        *self_rc.len_sample_header.borrow_mut() = _io.read_u4le()?;
        *self_rc.idx_sample_per_note.borrow_mut() = Vec::new();
        let l_idx_sample_per_note = 96;
        for _i in 0..l_idx_sample_per_note {
            self_rc.idx_sample_per_note.borrow_mut().push(_io.read_u1()?);
        }
        *self_rc.volume_points.borrow_mut() = Vec::new();
        let l_volume_points = 12;
        for _i in 0..l_volume_points {
            let t = Self::read_into::<_, FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.volume_points.borrow_mut().push(t);
        }
        *self_rc.panning_points.borrow_mut() = Vec::new();
        let l_panning_points = 12;
        for _i in 0..l_panning_points {
            let t = Self::read_into::<_, FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.panning_points.borrow_mut().push(t);
        }
        *self_rc.num_volume_points.borrow_mut() = _io.read_u1()?;
        *self_rc.num_panning_points.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_sustain_point.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_loop_start_point.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_loop_end_point.borrow_mut() = _io.read_u1()?;
        *self_rc.panning_sustain_point.borrow_mut() = _io.read_u1()?;
        *self_rc.panning_loop_start_point.borrow_mut() = _io.read_u1()?;
        *self_rc.panning_loop_end_point.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_type.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.panning_type.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.vibrato_type.borrow_mut() = _io.read_u1()?;
        *self_rc.vibrato_sweep.borrow_mut() = _io.read_u1()?;
        *self_rc.vibrato_depth.borrow_mut() = _io.read_u1()?;
        *self_rc.vibrato_rate.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_fadeout.borrow_mut() = _io.read_u2le()?;
        *self_rc.reserved.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn len_sample_header(&self) -> Ref<'_, u32> {
        self.len_sample_header.borrow()
    }
}

/**
 * Index of sample that should be used for any particular
 * note. In the simplest case, where it's only one sample
 * is available, it's an array of full of zeroes.
 */
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn idx_sample_per_note(&self) -> Ref<'_, Vec<u8>> {
        self.idx_sample_per_note.borrow()
    }
}

/**
 * Points for volume envelope. Only `num_volume_points` will be actually used.
 */
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn volume_points(&self) -> Ref<'_, Vec<OptRc<FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint>>> {
        self.volume_points.borrow()
    }
}

/**
 * Points for panning envelope. Only `num_panning_points` will be actually used.
 */
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn panning_points(&self) -> Ref<'_, Vec<OptRc<FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint>>> {
        self.panning_points.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn num_volume_points(&self) -> Ref<'_, u8> {
        self.num_volume_points.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn num_panning_points(&self) -> Ref<'_, u8> {
        self.num_panning_points.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn volume_sustain_point(&self) -> Ref<'_, u8> {
        self.volume_sustain_point.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn volume_loop_start_point(&self) -> Ref<'_, u8> {
        self.volume_loop_start_point.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn volume_loop_end_point(&self) -> Ref<'_, u8> {
        self.volume_loop_end_point.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn panning_sustain_point(&self) -> Ref<'_, u8> {
        self.panning_sustain_point.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn panning_loop_start_point(&self) -> Ref<'_, u8> {
        self.panning_loop_start_point.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn panning_loop_end_point(&self) -> Ref<'_, u8> {
        self.panning_loop_end_point.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn volume_type(&self) -> Ref<'_, FasttrackerXmModule_Instrument_ExtraHeader_Type> {
        self.volume_type.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn panning_type(&self) -> Ref<'_, FasttrackerXmModule_Instrument_ExtraHeader_Type> {
        self.panning_type.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn vibrato_type(&self) -> Ref<'_, u8> {
        self.vibrato_type.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn vibrato_sweep(&self) -> Ref<'_, u8> {
        self.vibrato_sweep.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn vibrato_depth(&self) -> Ref<'_, u8> {
        self.vibrato_depth.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn vibrato_rate(&self) -> Ref<'_, u8> {
        self.vibrato_rate.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn volume_fadeout(&self) -> Ref<'_, u16> {
        self.volume_fadeout.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn reserved(&self) -> Ref<'_, u16> {
        self.reserved.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum FasttrackerXmModule_Instrument_ExtraHeader_Type {
    On,
    Sustain,
    Loop,
    Unknown(i64),
}

impl TryFrom<i64> for FasttrackerXmModule_Instrument_ExtraHeader_Type {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<FasttrackerXmModule_Instrument_ExtraHeader_Type> {
        match flag {
            0 => Ok(FasttrackerXmModule_Instrument_ExtraHeader_Type::On),
            1 => Ok(FasttrackerXmModule_Instrument_ExtraHeader_Type::Sustain),
            2 => Ok(FasttrackerXmModule_Instrument_ExtraHeader_Type::Loop),
            _ => Ok(FasttrackerXmModule_Instrument_ExtraHeader_Type::Unknown(flag)),
        }
    }
}

impl From<&FasttrackerXmModule_Instrument_ExtraHeader_Type> for i64 {
    fn from(v: &FasttrackerXmModule_Instrument_ExtraHeader_Type) -> Self {
        match *v {
            FasttrackerXmModule_Instrument_ExtraHeader_Type::On => 0,
            FasttrackerXmModule_Instrument_ExtraHeader_Type::Sustain => 1,
            FasttrackerXmModule_Instrument_ExtraHeader_Type::Loop => 2,
            FasttrackerXmModule_Instrument_ExtraHeader_Type::Unknown(v) => v
        }
    }
}

impl Default for FasttrackerXmModule_Instrument_ExtraHeader_Type {
    fn default() -> Self { FasttrackerXmModule_Instrument_ExtraHeader_Type::Unknown(0) }
}


/**
 * Envelope frame-counters work in range 0..FFFFh (0..65535 dec).
 * BUT! FT2 only itself supports only range 0..FFh (0..255 dec).
 * Some other trackers (like SoundTracker for Unix), however, can use the full range 0..FFFF, so it should be supported.
 * !!TIP: This is also a good way to detect if the module has been made with FT2 or not. (In case the tracker name is brain- deadly left unchanged!)
 * Of course it does not help if all instruments have the values inside FT2 supported range.
 * The value-field of the envelope point is ranged between 00..3Fh (0..64 dec).
 */

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Instrument_ExtraHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<u16>,
    y: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Instrument_ExtraHeader;

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
        *self_rc.x.borrow_mut() = _io.read_u2le()?;
        *self_rc.y.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint {
}

/**
 * Frame number of the point
 */
impl FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint {
    pub fn x(&self) -> Ref<'_, u16> {
        self.x.borrow()
    }
}

/**
 * Value of the point
 */
impl FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint {
    pub fn y(&self) -> Ref<'_, u16> {
        self.y.borrow()
    }
}
impl FasttrackerXmModule_Instrument_ExtraHeader_EnvelopePoint {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument_Header {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Instrument>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    r#type: RefCell<u8>,
    num_samples: RefCell<u16>,
    extra_header: RefCell<OptRc<FasttrackerXmModule_Instrument_ExtraHeader>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument_Header {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Instrument;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(22_usize)?, 0, false), "UTF-8")?;
        *self_rc.r#type.borrow_mut() = _io.read_u1()?;
        *self_rc.num_samples.borrow_mut() = _io.read_u2le()?;
        if *self_rc.num_samples() > 0 {
            let t = Self::read_into::<_, FasttrackerXmModule_Instrument_ExtraHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.extra_header.borrow_mut() = t;
        }
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument_Header {
}
impl FasttrackerXmModule_Instrument_Header {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}

/**
 * Usually zero, but this seems pretty random, don't assume it's zero
 */
impl FasttrackerXmModule_Instrument_Header {
    pub fn r#type(&self) -> Ref<'_, u8> {
        self.r#type.borrow()
    }
}
impl FasttrackerXmModule_Instrument_Header {
    pub fn num_samples(&self) -> Ref<'_, u16> {
        self.num_samples.borrow()
    }
}
impl FasttrackerXmModule_Instrument_Header {
    pub fn extra_header(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Instrument_ExtraHeader>> {
        self.extra_header.borrow()
    }
}
impl FasttrackerXmModule_Instrument_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument_SampleHeader {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Instrument>,
    pub(crate) _self_shared: SharedType<Self>,
    sample_length: RefCell<u32>,
    sample_loop_start: RefCell<u32>,
    sample_loop_length: RefCell<u32>,
    volume: RefCell<u8>,
    fine_tune: RefCell<i8>,
    r#type: RefCell<OptRc<FasttrackerXmModule_Instrument_SampleHeader_LoopType>>,
    panning: RefCell<u8>,
    relative_note_number: RefCell<i8>,
    reserved: RefCell<u8>,
    name: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument_SampleHeader {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Instrument;

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
        *self_rc.sample_length.borrow_mut() = _io.read_u4le()?;
        *self_rc.sample_loop_start.borrow_mut() = _io.read_u4le()?;
        *self_rc.sample_loop_length.borrow_mut() = _io.read_u4le()?;
        *self_rc.volume.borrow_mut() = _io.read_u1()?;
        *self_rc.fine_tune.borrow_mut() = _io.read_s1()?;
        let t = Self::read_into::<_, FasttrackerXmModule_Instrument_SampleHeader_LoopType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.r#type.borrow_mut() = t;
        *self_rc.panning.borrow_mut() = _io.read_u1()?;
        *self_rc.relative_note_number.borrow_mut() = _io.read_s1()?;
        *self_rc.reserved.borrow_mut() = _io.read_u1()?;
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(22_usize)?, 0, false), "UTF-8")?;
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn sample_length(&self) -> Ref<'_, u32> {
        self.sample_length.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn sample_loop_start(&self) -> Ref<'_, u32> {
        self.sample_loop_start.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn sample_loop_length(&self) -> Ref<'_, u32> {
        self.sample_loop_length.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn volume(&self) -> Ref<'_, u8> {
        self.volume.borrow()
    }
}

/**
 * -16..+15
 */
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn fine_tune(&self) -> Ref<'_, i8> {
        self.fine_tune.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn r#type(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Instrument_SampleHeader_LoopType>> {
        self.r#type.borrow()
    }
}

/**
 * (0-255)
 */
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn panning(&self) -> Ref<'_, u8> {
        self.panning.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn relative_note_number(&self) -> Ref<'_, i8> {
        self.relative_note_number.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn reserved(&self) -> Ref<'_, u8> {
        self.reserved.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Instrument_SampleHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved0: RefCell<u64>,
    is_sample_data_16_bit: RefCell<bool>,
    reserved1: RefCell<u64>,
    loop_type: RefCell<FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Instrument_SampleHeader;

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
        *self_rc.reserved0.borrow_mut() = _io.read_bits_int_be(3)?;
        *self_rc.is_sample_data_16_bit.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(2)?;
        *self_rc.loop_type.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader_LoopType {
}
impl FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    pub fn reserved0(&self) -> Ref<'_, u64> {
        self.reserved0.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    pub fn is_sample_data_16_bit(&self) -> Ref<'_, bool> {
        self.is_sample_data_16_bit.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    pub fn reserved1(&self) -> Ref<'_, u64> {
        self.reserved1.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    pub fn loop_type(&self) -> Ref<'_, FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType> {
        self.loop_type.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SampleHeader_LoopType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType {
    None,
    Forward,
    PingPong,
    Unknown(i64),
}

impl TryFrom<i64> for FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType> {
        match flag {
            0 => Ok(FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::None),
            1 => Ok(FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::Forward),
            2 => Ok(FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::PingPong),
            _ => Ok(FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::Unknown(flag)),
        }
    }
}

impl From<&FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType> for i64 {
    fn from(v: &FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType) -> Self {
        match *v {
            FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::None => 0,
            FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::Forward => 1,
            FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::PingPong => 2,
            FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::Unknown(v) => v
        }
    }
}

impl Default for FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType {
    fn default() -> Self { FasttrackerXmModule_Instrument_SampleHeader_LoopType_LoopType::Unknown(0) }
}


/**
 * The saved data uses simple delta-encoding to achieve better compression ratios (when compressed with pkzip, etc.)
 * Pseudocode for converting the delta-coded data to normal data,
 * old = 0;
 * for i in range(data_len):
 *   new = sample[i] + old;
 *   sample[i] = new;
 *   old = new;
 */

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Instrument_SamplesData {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Instrument>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<FasttrackerXmModule_Instrument_SampleHeader>>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Instrument_SamplesData {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Instrument;

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
        *self_rc.data.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc.header().sample_length()).saturating_mul(u32::try_from(if *self_rc.header().r#type().is_sample_data_16_bit() { 2_i32 } else { 1_i32 })?))?)?;
        Ok(())
    }
}
impl FasttrackerXmModule_Instrument_SamplesData {
    pub fn header(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Instrument_SampleHeader>> {
        self.header.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SamplesData {
    pub fn set_params(&mut self, header: OptRc<FasttrackerXmModule_Instrument_SampleHeader>) {
        *self.header.borrow_mut() = header;
    }
}
impl FasttrackerXmModule_Instrument_SamplesData {
}
impl FasttrackerXmModule_Instrument_SamplesData {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl FasttrackerXmModule_Instrument_SamplesData {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Pattern {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<FasttrackerXmModule_Pattern_Header>>,
    packed_data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Pattern {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule;

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
        let t = Self::read_into::<_, FasttrackerXmModule_Pattern_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.packed_data.borrow_mut() = _io.read_bytes(usize::from(*self_rc.header().main().len_packed_pattern()))?;
        Ok(())
    }
}
impl FasttrackerXmModule_Pattern {
}
impl FasttrackerXmModule_Pattern {
    pub fn header(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Pattern_Header>> {
        self.header.borrow()
    }
}
impl FasttrackerXmModule_Pattern {
    pub fn packed_data(&self) -> Ref<'_, Vec<u8>> {
        self.packed_data.borrow()
    }
}
impl FasttrackerXmModule_Pattern {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Pattern_Header {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Pattern>,
    pub(crate) _self_shared: SharedType<Self>,
    header_length: RefCell<u32>,
    main: RefCell<OptRc<FasttrackerXmModule_Pattern_Header_HeaderMain>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Pattern_Header {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Pattern;

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
        *self_rc.header_length.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, FasttrackerXmModule_Pattern_Header_HeaderMain>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        Ok(())
    }
}
impl FasttrackerXmModule_Pattern_Header {
}

/**
 * Pattern header length
 */
impl FasttrackerXmModule_Pattern_Header {
    pub fn header_length(&self) -> Ref<'_, u32> {
        self.header_length.borrow()
    }
}
impl FasttrackerXmModule_Pattern_Header {
    pub fn main(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Pattern_Header_HeaderMain>> {
        self.main.borrow()
    }
}
impl FasttrackerXmModule_Pattern_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Pattern_Header_HeaderMain {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Pattern_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    packing_type: RefCell<u8>,
    num_rows_raw: RefCell<Option<FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw>>,
    len_packed_pattern: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_num_rows: Cell<bool>,
    num_rows: RefCell<i32>,
}
#[derive(Debug, Clone)]
pub enum FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw {
    U1(u8),
    U2(u16),
}
impl From<u8> for FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<&FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw> for u8 {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(e: &FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw) -> Self {
        if let FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw::U1(v) = e {
            return *v;
        }
        panic!("trying to convert from enum FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw::U1 to u8, enum value {:?}", e)
    }
}
impl From<u16> for FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl From<&FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw> for u16 {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(e: &FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw) -> Self {
        if let FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw::U2(v) = e {
            return *v;
        }
        panic!("trying to convert from enum FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw::U2 to u16, enum value {:?}", e)
    }
}
impl From<&FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw> for usize {
    fn from(e: &FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw) -> Self {
        match e {
            // Reason for fallback: invalid enum conversion to usize defaults to 0
            FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw::U1(v) => usize::try_from(*v).unwrap_or(0),
            // Reason for fallback: invalid enum conversion to usize defaults to 0
            FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw::U2(v) => usize::try_from(*v).unwrap_or(0),
        }
    }
}

impl KStruct for FasttrackerXmModule_Pattern_Header_HeaderMain {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Pattern_Header;

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
        *self_rc.packing_type.borrow_mut() = _io.read_u1()?;
        match *self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.preheader().version_number().value()? {
            258 => {
                *self_rc.num_rows_raw.borrow_mut() = Some(_io.read_u1()?.into());
            }
            _ => {
                *self_rc.num_rows_raw.borrow_mut() = Some(_io.read_u2le()?.into());
            }
        }
        *self_rc.len_packed_pattern.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl FasttrackerXmModule_Pattern_Header_HeaderMain {
    pub fn num_rows(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_num_rows.get() {
            return Ok(self.num_rows.borrow());
        }
        self.f_num_rows.set(true);
        *self.num_rows.borrow_mut() = ((i32::try_from(self.num_rows_raw())?).saturating_add(if *self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.preheader().version_number().value()? == 258 { 1_i32 } else { 0_i32 })).try_into()?;
        Ok(self.num_rows.borrow())
    }
}

/**
 * Packing type (always 0)
 */
impl FasttrackerXmModule_Pattern_Header_HeaderMain {
    pub fn packing_type(&self) -> Ref<'_, u8> {
        self.packing_type.borrow()
    }
}

/**
 * Number of rows in pattern (1..256)
 */
impl FasttrackerXmModule_Pattern_Header_HeaderMain {
    pub fn num_rows_raw(&self) -> usize {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.num_rows_raw.borrow().as_ref().map(|v| v.into()).unwrap_or(0)
    }
    pub fn num_rows_raw_enum(&self) -> Ref<'_, Option<FasttrackerXmModule_Pattern_Header_HeaderMain_NumRowsRaw>> {
        self.num_rows_raw.borrow()
    }
}

/**
 * Packed pattern data size
 */
impl FasttrackerXmModule_Pattern_Header_HeaderMain {
    pub fn len_packed_pattern(&self) -> Ref<'_, u16> {
        self.len_packed_pattern.borrow()
    }
}
impl FasttrackerXmModule_Pattern_Header_HeaderMain {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Preheader {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule>,
    pub(crate) _self_shared: SharedType<Self>,
    signature0: RefCell<Vec<u8>>,
    module_name: RefCell<String>,
    signature1: RefCell<Vec<u8>>,
    tracker_name: RefCell<String>,
    version_number: RefCell<OptRc<FasttrackerXmModule_Preheader_Version>>,
    header_size: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FasttrackerXmModule_Preheader {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule;

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
        *self_rc.signature0.borrow_mut() = _io.read_bytes(17_usize)?;
        if !(*self_rc.signature0() == vec![0x45u8, 0x78u8, 0x74u8, 0x65u8, 0x6eu8, 0x64u8, 0x65u8, 0x64u8, 0x20u8, 0x4du8, 0x6fu8, 0x64u8, 0x75u8, 0x6cu8, 0x65u8, 0x3au8, 0x20u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/preheader/seq/0".to_string() }));
        }
        *self_rc.module_name.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(20_usize)?, 0, false), "UTF-8")?;
        *self_rc.signature1.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.signature1() == vec![0x1au8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/preheader/seq/2".to_string() }));
        }
        *self_rc.tracker_name.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(20_usize)?, 0, false), "UTF-8")?;
        let t = Self::read_into::<_, FasttrackerXmModule_Preheader_Version>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.version_number.borrow_mut() = t;
        *self_rc.header_size.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl FasttrackerXmModule_Preheader {
}
impl FasttrackerXmModule_Preheader {
    pub fn signature0(&self) -> Ref<'_, Vec<u8>> {
        self.signature0.borrow()
    }
}

/**
 * Module name, padded with zeroes
 */
impl FasttrackerXmModule_Preheader {
    pub fn module_name(&self) -> Ref<'_, String> {
        self.module_name.borrow()
    }
}
impl FasttrackerXmModule_Preheader {
    pub fn signature1(&self) -> Ref<'_, Vec<u8>> {
        self.signature1.borrow()
    }
}

/**
 * Tracker name
 */
impl FasttrackerXmModule_Preheader {
    pub fn tracker_name(&self) -> Ref<'_, String> {
        self.tracker_name.borrow()
    }
}

/**
 * Format versions below [0x01, 0x04] have a LOT of differences. Check this field!
 */
impl FasttrackerXmModule_Preheader {
    pub fn version_number(&self) -> Ref<'_, OptRc<FasttrackerXmModule_Preheader_Version>> {
        self.version_number.borrow()
    }
}

/**
 * Header size << Calculated FROM THIS OFFSET, not from the beginning of the file! >>
 */
impl FasttrackerXmModule_Preheader {
    pub fn header_size(&self) -> Ref<'_, u32> {
        self.header_size.borrow()
    }
}
impl FasttrackerXmModule_Preheader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FasttrackerXmModule_Preheader_Version {
    pub(crate) _root: SharedType<FasttrackerXmModule>,
    pub(crate) _parent: SharedType<FasttrackerXmModule_Preheader>,
    pub(crate) _self_shared: SharedType<Self>,
    minor: RefCell<u8>,
    major: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<i32>,
}
impl KStruct for FasttrackerXmModule_Preheader_Version {
    type Root = FasttrackerXmModule;
    type Parent = FasttrackerXmModule_Preheader;

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
        *self_rc.minor.borrow_mut() = _io.read_u1()?;
        *self_rc.major.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl FasttrackerXmModule_Preheader_Version {
    pub fn value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = ((((*self.major()).wrapping_shl(8_u32)) | (i32::from(*self.minor())))).try_into()?;
        Ok(self.value.borrow())
    }
}

/**
 * currently 0x04
 */
impl FasttrackerXmModule_Preheader_Version {
    pub fn minor(&self) -> Ref<'_, u8> {
        self.minor.borrow()
    }
}

/**
 * currently 0x01
 */
impl FasttrackerXmModule_Preheader_Version {
    pub fn major(&self) -> Ref<'_, u8> {
        self.major.borrow()
    }
}
impl FasttrackerXmModule_Preheader_Version {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
