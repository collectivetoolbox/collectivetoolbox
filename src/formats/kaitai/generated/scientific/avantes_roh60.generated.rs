// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Avantes USB spectrometers are supplied with a Windows binary which
 * generates one ROH and one RCM file when the user clicks "Save
 * experiment". In the version of 6.0, the ROH file contains a header
 * of 22 four-byte floats, then the spectrum as a float array and a
 * footer of 3 floats. The first and last pixel numbers are specified in the
 * header and determine the (length+1) of the spectral data. In the tested
 * files, the length is (2032-211-1)=1820 pixels, but Kaitai determines this
 * automatically anyway.
 *
 * The wavelength calibration is stored as a polynomial with coefficients
 * of 'wlintercept', 'wlx1', ... 'wlx4', the argument of which is the
 * (pixel number + 1), as found out by comparing with the original
 * Avantes converted data files. There is no intensity calibration saved,
 * but it is recommended to do it in your program - the CCD in the spectrometer
 * is so uneven that one should prepare exact pixel-to-pixel calibration curves
 * to get reasonable spectral results.
 *
 * The rest of the header floats is not known to the author. Note that the
 * newer version of Avantes software has a different format, see also
 * <https://www.mathworks.com/matlabcentral/fileexchange/37103-avantes-to-matlab>
 *
 * The RCM file contains the user-specified comment, so it may be useful
 * for automatic conversion of data. You may wish to divide the spectra by
 * the integration time before comparing them.
 *
 * Written and tested by Filip Dominec, 2017-2018
 */

#[derive(Default, Debug, Clone)]
pub struct AvantesRoh60 {
    pub(crate) _root: SharedType<AvantesRoh60>,
    pub(crate) _parent: SharedType<AvantesRoh60>,
    pub(crate) _self_shared: SharedType<Self>,
    unknown1: RefCell<f32>,
    wlintercept: RefCell<f32>,
    wlx1: RefCell<f32>,
    wlx2: RefCell<f32>,
    wlx3: RefCell<f32>,
    wlx4: RefCell<f32>,
    unknown2: RefCell<Vec<f32>>,
    ipixfirst: RefCell<f32>,
    ipixlast: RefCell<f32>,
    unknown3: RefCell<Vec<f32>>,
    spectrum: RefCell<Vec<f32>>,
    integration_ms: RefCell<f32>,
    averaging: RefCell<f32>,
    pixel_smoothing: RefCell<f32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AvantesRoh60 {
    type Root = AvantesRoh60;
    type Parent = AvantesRoh60;

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
        *self_rc.unknown1.borrow_mut() = _io.read_f4le()?;
        *self_rc.wlintercept.borrow_mut() = _io.read_f4le()?;
        *self_rc.wlx1.borrow_mut() = _io.read_f4le()?;
        *self_rc.wlx2.borrow_mut() = _io.read_f4le()?;
        *self_rc.wlx3.borrow_mut() = _io.read_f4le()?;
        *self_rc.wlx4.borrow_mut() = _io.read_f4le()?;
        *self_rc.unknown2.borrow_mut() = Vec::new();
        let l_unknown2 = 9;
        for _i in 0..l_unknown2 {
            self_rc.unknown2.borrow_mut().push(_io.read_f4le()?);
        }
        *self_rc.ipixfirst.borrow_mut() = _io.read_f4le()?;
        *self_rc.ipixlast.borrow_mut() = _io.read_f4le()?;
        *self_rc.unknown3.borrow_mut() = Vec::new();
        let l_unknown3 = 4;
        for _i in 0..l_unknown3 {
            self_rc.unknown3.borrow_mut().push(_io.read_f4le()?);
        }
        *self_rc.spectrum.borrow_mut() = Vec::new();
        let l_spectrum = ((float_to_int(*self_rc.ipixlast())).saturating_sub(float_to_int(*self_rc.ipixfirst()))).saturating_sub(1_i32);
        for _i in 0..l_spectrum {
            self_rc.spectrum.borrow_mut().push(_io.read_f4le()?);
        }
        *self_rc.integration_ms.borrow_mut() = _io.read_f4le()?;
        *self_rc.averaging.borrow_mut() = _io.read_f4le()?;
        *self_rc.pixel_smoothing.borrow_mut() = _io.read_f4le()?;
        Ok(())
    }
}
impl AvantesRoh60 {
}
impl AvantesRoh60 {
    pub fn unknown1(&self) -> Ref<'_, f32> {
        self.unknown1.borrow()
    }
}
impl AvantesRoh60 {
    pub fn wlintercept(&self) -> Ref<'_, f32> {
        self.wlintercept.borrow()
    }
}
impl AvantesRoh60 {
    pub fn wlx1(&self) -> Ref<'_, f32> {
        self.wlx1.borrow()
    }
}
impl AvantesRoh60 {
    pub fn wlx2(&self) -> Ref<'_, f32> {
        self.wlx2.borrow()
    }
}
impl AvantesRoh60 {
    pub fn wlx3(&self) -> Ref<'_, f32> {
        self.wlx3.borrow()
    }
}
impl AvantesRoh60 {
    pub fn wlx4(&self) -> Ref<'_, f32> {
        self.wlx4.borrow()
    }
}
impl AvantesRoh60 {
    pub fn unknown2(&self) -> Ref<'_, Vec<f32>> {
        self.unknown2.borrow()
    }
}
impl AvantesRoh60 {
    pub fn ipixfirst(&self) -> Ref<'_, f32> {
        self.ipixfirst.borrow()
    }
}
impl AvantesRoh60 {
    pub fn ipixlast(&self) -> Ref<'_, f32> {
        self.ipixlast.borrow()
    }
}
impl AvantesRoh60 {
    pub fn unknown3(&self) -> Ref<'_, Vec<f32>> {
        self.unknown3.borrow()
    }
}
impl AvantesRoh60 {
    pub fn spectrum(&self) -> Ref<'_, Vec<f32>> {
        self.spectrum.borrow()
    }
}
impl AvantesRoh60 {
    pub fn integration_ms(&self) -> Ref<'_, f32> {
        self.integration_ms.borrow()
    }
}
impl AvantesRoh60 {
    pub fn averaging(&self) -> Ref<'_, f32> {
        self.averaging.borrow()
    }
}
impl AvantesRoh60 {
    pub fn pixel_smoothing(&self) -> Ref<'_, f32> {
        self.pixel_smoothing.borrow()
    }
}
impl AvantesRoh60 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
