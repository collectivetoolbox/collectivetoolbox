// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<ShapefileMain_FileHeader>>,
    records: RefCell<Vec<OptRc<ShapefileMain_Record>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain {
    type Root = ShapefileMain;
    type Parent = ShapefileMain;

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
        let t = Self::read_into::<_, ShapefileMain_FileHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.records.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, ShapefileMain_Record>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl ShapefileMain {
}
impl ShapefileMain {
    pub fn header(&self) -> Ref<'_, OptRc<ShapefileMain_FileHeader>> {
        self.header.borrow()
    }
}

/**
 * the size of this section of the file in bytes must equal (header.file_length * 2) - 100
 */
impl ShapefileMain {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Record>>> {
        self.records.borrow()
    }
}
impl ShapefileMain {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum ShapefileMain_PartType {
    TriangleStrip,
    TriangleFan,
    OuterRing,
    InnerRing,
    FirstRing,
    Ring,
    Unknown(i64),
}

impl TryFrom<i64> for ShapefileMain_PartType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ShapefileMain_PartType> {
        match flag {
            0 => Ok(ShapefileMain_PartType::TriangleStrip),
            1 => Ok(ShapefileMain_PartType::TriangleFan),
            2 => Ok(ShapefileMain_PartType::OuterRing),
            3 => Ok(ShapefileMain_PartType::InnerRing),
            4 => Ok(ShapefileMain_PartType::FirstRing),
            5 => Ok(ShapefileMain_PartType::Ring),
            _ => Ok(ShapefileMain_PartType::Unknown(flag)),
        }
    }
}

impl From<&ShapefileMain_PartType> for i64 {
    fn from(v: &ShapefileMain_PartType) -> Self {
        match *v {
            ShapefileMain_PartType::TriangleStrip => 0,
            ShapefileMain_PartType::TriangleFan => 1,
            ShapefileMain_PartType::OuterRing => 2,
            ShapefileMain_PartType::InnerRing => 3,
            ShapefileMain_PartType::FirstRing => 4,
            ShapefileMain_PartType::Ring => 5,
            ShapefileMain_PartType::Unknown(v) => v
        }
    }
}

impl Default for ShapefileMain_PartType {
    fn default() -> Self { ShapefileMain_PartType::Unknown(0) }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ShapefileMain_ShapeType {
    NullShape,
    Point,
    PolyLine,
    Polygon,
    MultiPoint,
    PointZ,
    PolyLineZ,
    PolygonZ,
    MultiPointZ,
    PointM,
    PolyLineM,
    PolygonM,
    MultiPointM,
    MultiPatch,
    Unknown(i64),
}

impl TryFrom<i64> for ShapefileMain_ShapeType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ShapefileMain_ShapeType> {
        match flag {
            0 => Ok(ShapefileMain_ShapeType::NullShape),
            1 => Ok(ShapefileMain_ShapeType::Point),
            3 => Ok(ShapefileMain_ShapeType::PolyLine),
            5 => Ok(ShapefileMain_ShapeType::Polygon),
            8 => Ok(ShapefileMain_ShapeType::MultiPoint),
            11 => Ok(ShapefileMain_ShapeType::PointZ),
            13 => Ok(ShapefileMain_ShapeType::PolyLineZ),
            15 => Ok(ShapefileMain_ShapeType::PolygonZ),
            18 => Ok(ShapefileMain_ShapeType::MultiPointZ),
            21 => Ok(ShapefileMain_ShapeType::PointM),
            23 => Ok(ShapefileMain_ShapeType::PolyLineM),
            25 => Ok(ShapefileMain_ShapeType::PolygonM),
            28 => Ok(ShapefileMain_ShapeType::MultiPointM),
            31 => Ok(ShapefileMain_ShapeType::MultiPatch),
            _ => Ok(ShapefileMain_ShapeType::Unknown(flag)),
        }
    }
}

impl From<&ShapefileMain_ShapeType> for i64 {
    fn from(v: &ShapefileMain_ShapeType) -> Self {
        match *v {
            ShapefileMain_ShapeType::NullShape => 0,
            ShapefileMain_ShapeType::Point => 1,
            ShapefileMain_ShapeType::PolyLine => 3,
            ShapefileMain_ShapeType::Polygon => 5,
            ShapefileMain_ShapeType::MultiPoint => 8,
            ShapefileMain_ShapeType::PointZ => 11,
            ShapefileMain_ShapeType::PolyLineZ => 13,
            ShapefileMain_ShapeType::PolygonZ => 15,
            ShapefileMain_ShapeType::MultiPointZ => 18,
            ShapefileMain_ShapeType::PointM => 21,
            ShapefileMain_ShapeType::PolyLineM => 23,
            ShapefileMain_ShapeType::PolygonM => 25,
            ShapefileMain_ShapeType::MultiPointM => 28,
            ShapefileMain_ShapeType::MultiPatch => 31,
            ShapefileMain_ShapeType::Unknown(v) => v
        }
    }
}

impl Default for ShapefileMain_ShapeType {
    fn default() -> Self { ShapefileMain_ShapeType::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_BoundingBoxXY {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    y: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_BoundingBoxXY {
    type Root = ShapefileMain;
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
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.x.borrow_mut() = t;
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.y.borrow_mut() = t;
        Ok(())
    }
}
impl ShapefileMain_BoundingBoxXY {
}
impl ShapefileMain_BoundingBoxXY {
    pub fn x(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.x.borrow()
    }
}
impl ShapefileMain_BoundingBoxXY {
    pub fn y(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.y.borrow()
    }
}
impl ShapefileMain_BoundingBoxXY {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_BoundingBoxXYZM {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_FileHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    y: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    z: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_BoundingBoxXYZM {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_FileHeader;

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
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.x.borrow_mut() = t;
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.y.borrow_mut() = t;
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.z.borrow_mut() = t;
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m.borrow_mut() = t;
        Ok(())
    }
}
impl ShapefileMain_BoundingBoxXYZM {
}
impl ShapefileMain_BoundingBoxXYZM {
    pub fn x(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.x.borrow()
    }
}
impl ShapefileMain_BoundingBoxXYZM {
    pub fn y(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.y.borrow()
    }
}
impl ShapefileMain_BoundingBoxXYZM {
    pub fn z(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.z.borrow()
    }
}
impl ShapefileMain_BoundingBoxXYZM {
    pub fn m(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m.borrow()
    }
}
impl ShapefileMain_BoundingBoxXYZM {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_BoundsMinMax {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    min: RefCell<f64>,
    max: RefCell<f64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_BoundsMinMax {
    type Root = ShapefileMain;
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
        *self_rc.min.borrow_mut() = _io.read_f8le()?;
        *self_rc.max.borrow_mut() = _io.read_f8le()?;
        Ok(())
    }
}
impl ShapefileMain_BoundsMinMax {
}
impl ShapefileMain_BoundsMinMax {
    pub fn min(&self) -> Ref<'_, f64> {
        self.min.borrow()
    }
}
impl ShapefileMain_BoundsMinMax {
    pub fn max(&self) -> Ref<'_, f64> {
        self.max.borrow()
    }
}
impl ShapefileMain_BoundsMinMax {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_FileHeader {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain>,
    pub(crate) _self_shared: SharedType<Self>,
    file_code: RefCell<Vec<u8>>,
    unused_field_1: RefCell<Vec<u8>>,
    unused_field_2: RefCell<Vec<u8>>,
    unused_field_3: RefCell<Vec<u8>>,
    unused_field_4: RefCell<Vec<u8>>,
    unused_field_5: RefCell<Vec<u8>>,
    file_length: RefCell<i32>,
    version: RefCell<Vec<u8>>,
    shape_type: RefCell<ShapefileMain_ShapeType>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXYZM>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_FileHeader {
    type Root = ShapefileMain;
    type Parent = ShapefileMain;

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
        *self_rc.file_code.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.file_code() == vec![0x0u8, 0x0u8, 0x27u8, 0xau8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/0".to_string() }));
        }
        *self_rc.unused_field_1.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.unused_field_1() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/1".to_string() }));
        }
        *self_rc.unused_field_2.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.unused_field_2() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/2".to_string() }));
        }
        *self_rc.unused_field_3.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.unused_field_3() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/3".to_string() }));
        }
        *self_rc.unused_field_4.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.unused_field_4() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/4".to_string() }));
        }
        *self_rc.unused_field_5.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.unused_field_5() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/5".to_string() }));
        }
        *self_rc.file_length.borrow_mut() = _io.read_s4be()?;
        *self_rc.version.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.version() == vec![0xe8u8, 0x3u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/7".to_string() }));
        }
        *self_rc.shape_type.borrow_mut() = i64::from(_io.read_s4le()?).try_into()?;
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXYZM>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        Ok(())
    }
}
impl ShapefileMain_FileHeader {
}

/**
 * corresponds to s4be value of 9994
 */
impl ShapefileMain_FileHeader {
    pub fn file_code(&self) -> Ref<'_, Vec<u8>> {
        self.file_code.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn unused_field_1(&self) -> Ref<'_, Vec<u8>> {
        self.unused_field_1.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn unused_field_2(&self) -> Ref<'_, Vec<u8>> {
        self.unused_field_2.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn unused_field_3(&self) -> Ref<'_, Vec<u8>> {
        self.unused_field_3.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn unused_field_4(&self) -> Ref<'_, Vec<u8>> {
        self.unused_field_4.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn unused_field_5(&self) -> Ref<'_, Vec<u8>> {
        self.unused_field_5.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn file_length(&self) -> Ref<'_, i32> {
        self.file_length.borrow()
    }
}

/**
 * corresponds to s4le value of 1000
 */
impl ShapefileMain_FileHeader {
    pub fn version(&self) -> Ref<'_, Vec<u8>> {
        self.version.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn shape_type(&self) -> Ref<'_, ShapefileMain_ShapeType> {
        self.shape_type.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXYZM>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_FileHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_MultiPatch {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    part_types: RefCell<Vec<ShapefileMain_PartType>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    z_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    z_values: RefCell<Vec<f64>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_MultiPatch {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.part_types.borrow_mut() = Vec::new();
        let l_part_types = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_part_types {
            self_rc.part_types.borrow_mut().push(i64::from(_io.read_s4le()?).try_into()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.z_range.borrow_mut() = t;
        *self_rc.z_values.borrow_mut() = Vec::new();
        let l_z_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_z_values {
            self_rc.z_values.borrow_mut().push(_io.read_f8le()?);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_MultiPatch {
}
impl ShapefileMain_MultiPatch {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn part_types(&self) -> Ref<'_, Vec<ShapefileMain_PartType>> {
        self.part_types.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn z_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.z_range.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn z_values(&self) -> Ref<'_, Vec<f64>> {
        self.z_values.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_MultiPatch {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_MultiPoint {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_points: RefCell<i32>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_MultiPoint {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl ShapefileMain_MultiPoint {
}
impl ShapefileMain_MultiPoint {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_MultiPoint {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_MultiPoint {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_MultiPoint {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_MultiPointM {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_points: RefCell<i32>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_MultiPointM {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_MultiPointM {
}
impl ShapefileMain_MultiPointM {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_MultiPointM {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_MultiPointM {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_MultiPointM {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_MultiPointM {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_MultiPointM {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_MultiPointZ {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_points: RefCell<i32>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    z_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    z_values: RefCell<Vec<f64>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_MultiPointZ {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.z_range.borrow_mut() = t;
        *self_rc.z_values.borrow_mut() = Vec::new();
        let l_z_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_z_values {
            self_rc.z_values.borrow_mut().push(_io.read_f8le()?);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_MultiPointZ {
}
impl ShapefileMain_MultiPointZ {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn z_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.z_range.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn z_values(&self) -> Ref<'_, Vec<f64>> {
        self.z_values.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_MultiPointZ {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_Point {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<f64>,
    y: RefCell<f64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_Point {
    type Root = ShapefileMain;
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
        *self_rc.x.borrow_mut() = _io.read_f8le()?;
        *self_rc.y.borrow_mut() = _io.read_f8le()?;
        Ok(())
    }
}
impl ShapefileMain_Point {
}
impl ShapefileMain_Point {
    pub fn x(&self) -> Ref<'_, f64> {
        self.x.borrow()
    }
}
impl ShapefileMain_Point {
    pub fn y(&self) -> Ref<'_, f64> {
        self.y.borrow()
    }
}
impl ShapefileMain_Point {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PointM {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<f64>,
    y: RefCell<f64>,
    m: RefCell<f64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PointM {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        *self_rc.x.borrow_mut() = _io.read_f8le()?;
        *self_rc.y.borrow_mut() = _io.read_f8le()?;
        *self_rc.m.borrow_mut() = _io.read_f8le()?;
        Ok(())
    }
}
impl ShapefileMain_PointM {
}
impl ShapefileMain_PointM {
    pub fn x(&self) -> Ref<'_, f64> {
        self.x.borrow()
    }
}
impl ShapefileMain_PointM {
    pub fn y(&self) -> Ref<'_, f64> {
        self.y.borrow()
    }
}
impl ShapefileMain_PointM {
    pub fn m(&self) -> Ref<'_, f64> {
        self.m.borrow()
    }
}
impl ShapefileMain_PointM {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PointZ {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    x: RefCell<f64>,
    y: RefCell<f64>,
    z: RefCell<f64>,
    m: RefCell<f64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PointZ {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        *self_rc.x.borrow_mut() = _io.read_f8le()?;
        *self_rc.y.borrow_mut() = _io.read_f8le()?;
        *self_rc.z.borrow_mut() = _io.read_f8le()?;
        *self_rc.m.borrow_mut() = _io.read_f8le()?;
        Ok(())
    }
}
impl ShapefileMain_PointZ {
}
impl ShapefileMain_PointZ {
    pub fn x(&self) -> Ref<'_, f64> {
        self.x.borrow()
    }
}
impl ShapefileMain_PointZ {
    pub fn y(&self) -> Ref<'_, f64> {
        self.y.borrow()
    }
}
impl ShapefileMain_PointZ {
    pub fn z(&self) -> Ref<'_, f64> {
        self.z.borrow()
    }
}
impl ShapefileMain_PointZ {
    pub fn m(&self) -> Ref<'_, f64> {
        self.m.borrow()
    }
}
impl ShapefileMain_PointZ {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PolyLine {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PolyLine {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl ShapefileMain_PolyLine {
}
impl ShapefileMain_PolyLine {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_PolyLine {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_PolyLine {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_PolyLine {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_PolyLine {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_PolyLine {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PolyLineM {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PolyLineM {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_PolyLineM {
}
impl ShapefileMain_PolyLineM {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_PolyLineM {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PolyLineZ {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    z_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    z_values: RefCell<Vec<f64>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PolyLineZ {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.z_range.borrow_mut() = t;
        *self_rc.z_values.borrow_mut() = Vec::new();
        let l_z_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_z_values {
            self_rc.z_values.borrow_mut().push(_io.read_f8le()?);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_PolyLineZ {
}
impl ShapefileMain_PolyLineZ {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn z_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.z_range.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn z_values(&self) -> Ref<'_, Vec<f64>> {
        self.z_values.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_PolyLineZ {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_Polygon {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_Polygon {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl ShapefileMain_Polygon {
}
impl ShapefileMain_Polygon {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_Polygon {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_Polygon {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_Polygon {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_Polygon {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_Polygon {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PolygonM {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PolygonM {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_PolygonM {
}
impl ShapefileMain_PolygonM {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_PolygonM {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_PolygonZ {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_RecordContents>,
    pub(crate) _self_shared: SharedType<Self>,
    bounding_box: RefCell<OptRc<ShapefileMain_BoundingBoxXY>>,
    number_of_parts: RefCell<i32>,
    number_of_points: RefCell<i32>,
    parts: RefCell<Vec<i32>>,
    points: RefCell<Vec<OptRc<ShapefileMain_Point>>>,
    z_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    z_values: RefCell<Vec<f64>>,
    m_range: RefCell<OptRc<ShapefileMain_BoundsMinMax>>,
    m_values: RefCell<Vec<f64>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_PolygonZ {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_RecordContents;

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
        let t = Self::read_into::<_, ShapefileMain_BoundingBoxXY>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bounding_box.borrow_mut() = t;
        *self_rc.number_of_parts.borrow_mut() = _io.read_s4le()?;
        *self_rc.number_of_points.borrow_mut() = _io.read_s4le()?;
        *self_rc.parts.borrow_mut() = Vec::new();
        let l_parts = usize::try_from(*self_rc.number_of_parts())?;
        for _i in 0_usize..l_parts {
            self_rc.parts.borrow_mut().push(_io.read_s4le()?);
        }
        *self_rc.points.borrow_mut() = Vec::new();
        let l_points = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_points {
            let t = Self::read_into::<_, ShapefileMain_Point>(&*_io, Some(self_rc._root.clone()), None)?.into();
            self_rc.points.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.z_range.borrow_mut() = t;
        *self_rc.z_values.borrow_mut() = Vec::new();
        let l_z_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_z_values {
            self_rc.z_values.borrow_mut().push(_io.read_f8le()?);
        }
        let t = Self::read_into::<_, ShapefileMain_BoundsMinMax>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.m_range.borrow_mut() = t;
        *self_rc.m_values.borrow_mut() = Vec::new();
        let l_m_values = usize::try_from(*self_rc.number_of_points())?;
        for _i in 0_usize..l_m_values {
            self_rc.m_values.borrow_mut().push(_io.read_f8le()?);
        }
        Ok(())
    }
}
impl ShapefileMain_PolygonZ {
}
impl ShapefileMain_PolygonZ {
    pub fn bounding_box(&self) -> Ref<'_, OptRc<ShapefileMain_BoundingBoxXY>> {
        self.bounding_box.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn number_of_parts(&self) -> Ref<'_, i32> {
        self.number_of_parts.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn number_of_points(&self) -> Ref<'_, i32> {
        self.number_of_points.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn parts(&self) -> Ref<'_, Vec<i32>> {
        self.parts.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn points(&self) -> Ref<'_, Vec<OptRc<ShapefileMain_Point>>> {
        self.points.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn z_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.z_range.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn z_values(&self) -> Ref<'_, Vec<f64>> {
        self.z_values.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn m_range(&self) -> Ref<'_, OptRc<ShapefileMain_BoundsMinMax>> {
        self.m_range.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn m_values(&self) -> Ref<'_, Vec<f64>> {
        self.m_values.borrow()
    }
}
impl ShapefileMain_PolygonZ {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_Record {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<ShapefileMain_RecordHeader>>,
    contents: RefCell<OptRc<ShapefileMain_RecordContents>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_Record {
    type Root = ShapefileMain;
    type Parent = ShapefileMain;

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
        let t = Self::read_into::<_, ShapefileMain_RecordHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        let t = Self::read_into::<_, ShapefileMain_RecordContents>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.contents.borrow_mut() = t;
        Ok(())
    }
}
impl ShapefileMain_Record {
}
impl ShapefileMain_Record {
    pub fn header(&self) -> Ref<'_, OptRc<ShapefileMain_RecordHeader>> {
        self.header.borrow()
    }
}

/**
 * the size of this contents section in bytes must equal header.content_length * 2
 */
impl ShapefileMain_Record {
    pub fn contents(&self) -> Ref<'_, OptRc<ShapefileMain_RecordContents>> {
        self.contents.borrow()
    }
}
impl ShapefileMain_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_RecordContents {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    shape_type: RefCell<ShapefileMain_ShapeType>,
    shape_parameters: RefCell<Option<ShapefileMain_RecordContents_ShapeParameters>>,
    _io: RefCell<BytesReader>,
    shape_parameters_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum ShapefileMain_RecordContents_ShapeParameters {
    ShapefileMain_MultiPatch(OptRc<ShapefileMain_MultiPatch>),
    ShapefileMain_MultiPoint(OptRc<ShapefileMain_MultiPoint>),
    ShapefileMain_MultiPointM(OptRc<ShapefileMain_MultiPointM>),
    ShapefileMain_MultiPointZ(OptRc<ShapefileMain_MultiPointZ>),
    ShapefileMain_Point(OptRc<ShapefileMain_Point>),
    ShapefileMain_PointM(OptRc<ShapefileMain_PointM>),
    ShapefileMain_PointZ(OptRc<ShapefileMain_PointZ>),
    ShapefileMain_PolyLine(OptRc<ShapefileMain_PolyLine>),
    ShapefileMain_PolyLineM(OptRc<ShapefileMain_PolyLineM>),
    ShapefileMain_PolyLineZ(OptRc<ShapefileMain_PolyLineZ>),
    ShapefileMain_Polygon(OptRc<ShapefileMain_Polygon>),
    ShapefileMain_PolygonM(OptRc<ShapefileMain_PolygonM>),
    ShapefileMain_PolygonZ(OptRc<ShapefileMain_PolygonZ>),
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_MultiPatch> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPatch(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPatch, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_MultiPatch>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_MultiPatch>) -> Self {
        Self::ShapefileMain_MultiPatch(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_MultiPoint> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPoint(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPoint, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_MultiPoint>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_MultiPoint>) -> Self {
        Self::ShapefileMain_MultiPoint(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_MultiPointM> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPointM(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPointM, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_MultiPointM>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_MultiPointM>) -> Self {
        Self::ShapefileMain_MultiPointM(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_MultiPointZ> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPointZ(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_MultiPointZ, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_MultiPointZ>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_MultiPointZ>) -> Self {
        Self::ShapefileMain_MultiPointZ(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_Point> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_Point(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_Point, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_Point>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_Point>) -> Self {
        Self::ShapefileMain_Point(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PointM> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PointM(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PointM, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PointM>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PointM>) -> Self {
        Self::ShapefileMain_PointM(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PointZ> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PointZ(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PointZ, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PointZ>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PointZ>) -> Self {
        Self::ShapefileMain_PointZ(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PolyLine> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolyLine(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolyLine, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PolyLine>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PolyLine>) -> Self {
        Self::ShapefileMain_PolyLine(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PolyLineM> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolyLineM(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolyLineM, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PolyLineM>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PolyLineM>) -> Self {
        Self::ShapefileMain_PolyLineM(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PolyLineZ> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolyLineZ(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolyLineZ, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PolyLineZ>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PolyLineZ>) -> Self {
        Self::ShapefileMain_PolyLineZ(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_Polygon> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_Polygon(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_Polygon, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_Polygon>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_Polygon>) -> Self {
        Self::ShapefileMain_Polygon(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PolygonM> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolygonM(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolygonM, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PolygonM>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PolygonM>) -> Self {
        Self::ShapefileMain_PolygonM(v)
    }
}
impl From<&ShapefileMain_RecordContents_ShapeParameters> for OptRc<ShapefileMain_PolygonZ> {
    #[allow(clippy::panic, reason = "Fallible Kaitai switch-type variant conversion")]
    fn from(v: &ShapefileMain_RecordContents_ShapeParameters) -> Self {
        if let ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolygonZ(x) = v {
            return x.clone();
        }
        panic!("expected ShapefileMain_RecordContents_ShapeParameters::ShapefileMain_PolygonZ, got {:?}", v)
    }
}
impl From<OptRc<ShapefileMain_PolygonZ>> for ShapefileMain_RecordContents_ShapeParameters {
    fn from(v: OptRc<ShapefileMain_PolygonZ>) -> Self {
        Self::ShapefileMain_PolygonZ(v)
    }
}
impl KStruct for ShapefileMain_RecordContents {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_Record;

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
        *self_rc.shape_type.borrow_mut() = i64::from(_io.read_s4le()?).try_into()?;
        if *self_rc.shape_type() != ShapefileMain_ShapeType::NullShape {
            match *self_rc.shape_type() {
                ShapefileMain_ShapeType::MultiPatch => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_MultiPatch>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::MultiPoint => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_MultiPoint>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::MultiPointM => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_MultiPointM>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::MultiPointZ => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_MultiPointZ>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::Point => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_Point>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), None)?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PointM => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PointM>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PointZ => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PointZ>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PolyLine => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PolyLine>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PolyLineM => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PolyLineM>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PolyLineZ => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PolyLineZ>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::Polygon => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_Polygon>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PolygonM => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PolygonM>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                ShapefileMain_ShapeType::PolygonZ => {
                    *self_rc.shape_parameters_raw.borrow_mut() = _io.read_bytes_full()?.into();
                    let shape_parameters_raw = self_rc.shape_parameters_raw.borrow();
                    let _t_shape_parameters_raw_io = BytesReader::from(shape_parameters_raw.clone());
                    let t = Self::read_into::<BytesReader, ShapefileMain_PolygonZ>(&_t_shape_parameters_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                    *self_rc.shape_parameters.borrow_mut() = Some(t);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
impl ShapefileMain_RecordContents {
}
impl ShapefileMain_RecordContents {
    pub fn shape_type(&self) -> Ref<'_, ShapefileMain_ShapeType> {
        self.shape_type.borrow()
    }
}
impl ShapefileMain_RecordContents {
    pub fn shape_parameters(&self) -> Ref<'_, Option<ShapefileMain_RecordContents_ShapeParameters>> {
        self.shape_parameters.borrow()
    }
}
impl ShapefileMain_RecordContents {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ShapefileMain_RecordContents {
    pub fn shape_parameters_raw(&self) -> Ref<'_, Vec<u8>> {
        self.shape_parameters_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShapefileMain_RecordHeader {
    pub(crate) _root: SharedType<ShapefileMain>,
    pub(crate) _parent: SharedType<ShapefileMain_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    record_number: RefCell<i32>,
    content_length: RefCell<i32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ShapefileMain_RecordHeader {
    type Root = ShapefileMain;
    type Parent = ShapefileMain_Record;

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
        *self_rc.record_number.borrow_mut() = _io.read_s4be()?;
        *self_rc.content_length.borrow_mut() = _io.read_s4be()?;
        Ok(())
    }
}
impl ShapefileMain_RecordHeader {
}
impl ShapefileMain_RecordHeader {
    pub fn record_number(&self) -> Ref<'_, i32> {
        self.record_number.borrow()
    }
}
impl ShapefileMain_RecordHeader {
    pub fn content_length(&self) -> Ref<'_, i32> {
        self.content_length.borrow()
    }
}
impl ShapefileMain_RecordHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
