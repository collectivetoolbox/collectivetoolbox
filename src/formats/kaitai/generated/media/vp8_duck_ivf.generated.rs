// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Duck IVF is a simple container format for raw VP8 data, which is an open and
 * royalty-free video compression format, currently developed by Google.
 *
 * Test .ivf files are available at
 * <https://chromium.googlesource.com/webm/vp8-test-vectors>
 * \sa <https://wiki.multimedia.cx/index.php/Duck_IVF> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Vp8DuckIvf {
    pub(crate) _root: SharedType<Vp8DuckIvf>,
    pub(crate) _parent: SharedType<Vp8DuckIvf>,
    pub(crate) _self_shared: SharedType<Self>,
    magic1: RefCell<Vec<u8>>,
    version: RefCell<u16>,
    len_header: RefCell<u16>,
    codec: RefCell<Vec<u8>>,
    width: RefCell<u16>,
    height: RefCell<u16>,
    framerate: RefCell<u32>,
    timescale: RefCell<u32>,
    num_frames: RefCell<u32>,
    unused: RefCell<u32>,
    image_data: RefCell<Vec<OptRc<Vp8DuckIvf_Blocks>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vp8DuckIvf {
    type Root = Vp8DuckIvf;
    type Parent = Vp8DuckIvf;

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
        *self_rc.magic1.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.magic1() == vec![0x44u8, 0x4bu8, 0x49u8, 0x46u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.version.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_header.borrow_mut() = _io.read_u2le()?;
        *self_rc.codec.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.codec() == vec![0x56u8, 0x50u8, 0x38u8, 0x30u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/3".to_string() }));
        }
        *self_rc.width.borrow_mut() = _io.read_u2le()?;
        *self_rc.height.borrow_mut() = _io.read_u2le()?;
        *self_rc.framerate.borrow_mut() = _io.read_u4le()?;
        *self_rc.timescale.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_frames.borrow_mut() = _io.read_u4le()?;
        *self_rc.unused.borrow_mut() = _io.read_u4le()?;
        *self_rc.image_data.borrow_mut() = Vec::new();
        let l_image_data = usize::try_from(*self_rc.num_frames())?;
        for _i in 0_usize..l_image_data {
            let t = Self::read_into::<_, Vp8DuckIvf_Blocks>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.image_data.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl Vp8DuckIvf {
}

/**
 * Magic Number of IVF Files
 */
impl Vp8DuckIvf {
    pub fn magic1(&self) -> Ref<'_, Vec<u8>> {
        self.magic1.borrow()
    }
}

/**
 * This should be 0
 */
impl Vp8DuckIvf {
    pub fn version(&self) -> Ref<'_, u16> {
        self.version.borrow()
    }
}

/**
 * Normally the header length is 32 byte
 */
impl Vp8DuckIvf {
    pub fn len_header(&self) -> Ref<'_, u16> {
        self.len_header.borrow()
    }
}

/**
 * Name of the codec e.g. 'VP80' for VP8
 */
impl Vp8DuckIvf {
    pub fn codec(&self) -> Ref<'_, Vec<u8>> {
        self.codec.borrow()
    }
}

/**
 * The (initial) width of the video, every keyframe may change the resolution
 */
impl Vp8DuckIvf {
    pub fn width(&self) -> Ref<'_, u16> {
        self.width.borrow()
    }
}

/**
 * The (initial) height of the video, every keyframe may change the resolution
 */
impl Vp8DuckIvf {
    pub fn height(&self) -> Ref<'_, u16> {
        self.height.borrow()
    }
}

/**
 * the (framerate * timescale) e.g. for 30 fps -> 30000
 */
impl Vp8DuckIvf {
    pub fn framerate(&self) -> Ref<'_, u32> {
        self.framerate.borrow()
    }
}

/**
 * the timescale is a divider of the seconds (VPX is integer math only) mostly 1000
 */
impl Vp8DuckIvf {
    pub fn timescale(&self) -> Ref<'_, u32> {
        self.timescale.borrow()
    }
}

/**
 * the number of frames (if not a camera stream)
 */
impl Vp8DuckIvf {
    pub fn num_frames(&self) -> Ref<'_, u32> {
        self.num_frames.borrow()
    }
}
impl Vp8DuckIvf {
    pub fn unused(&self) -> Ref<'_, u32> {
        self.unused.borrow()
    }
}
impl Vp8DuckIvf {
    pub fn image_data(&self) -> Ref<'_, Vec<OptRc<Vp8DuckIvf_Blocks>>> {
        self.image_data.borrow()
    }
}
impl Vp8DuckIvf {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vp8DuckIvf_Block {
    pub(crate) _root: SharedType<Vp8DuckIvf>,
    pub(crate) _parent: SharedType<Vp8DuckIvf_Blocks>,
    pub(crate) _self_shared: SharedType<Self>,
    len_frame: RefCell<u32>,
    timestamp: RefCell<u64>,
    framedata: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vp8DuckIvf_Block {
    type Root = Vp8DuckIvf;
    type Parent = Vp8DuckIvf_Blocks;

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
        *self_rc.len_frame.borrow_mut() = _io.read_u4le()?;
        *self_rc.timestamp.borrow_mut() = _io.read_u8le()?;
        *self_rc.framedata.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_frame())?)?;
        Ok(())
    }
}
impl Vp8DuckIvf_Block {
}

/**
 * size of the frame data
 */
impl Vp8DuckIvf_Block {
    pub fn len_frame(&self) -> Ref<'_, u32> {
        self.len_frame.borrow()
    }
}
impl Vp8DuckIvf_Block {
    pub fn timestamp(&self) -> Ref<'_, u64> {
        self.timestamp.borrow()
    }
}
impl Vp8DuckIvf_Block {
    pub fn framedata(&self) -> Ref<'_, Vec<u8>> {
        self.framedata.borrow()
    }
}
impl Vp8DuckIvf_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vp8DuckIvf_Blocks {
    pub(crate) _root: SharedType<Vp8DuckIvf>,
    pub(crate) _parent: SharedType<Vp8DuckIvf>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<OptRc<Vp8DuckIvf_Block>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vp8DuckIvf_Blocks {
    type Root = Vp8DuckIvf;
    type Parent = Vp8DuckIvf;

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
        let t = Self::read_into::<_, Vp8DuckIvf_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.entries.borrow_mut() = t;
        Ok(())
    }
}
impl Vp8DuckIvf_Blocks {
}
impl Vp8DuckIvf_Blocks {
    pub fn entries(&self) -> Ref<'_, OptRc<Vp8DuckIvf_Block>> {
        self.entries.borrow()
    }
}
impl Vp8DuckIvf_Blocks {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
