// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct CpioOldLe {
    pub(crate) _root: SharedType<CpioOldLe>,
    pub(crate) _parent: SharedType<CpioOldLe>,
    pub(crate) _self_shared: SharedType<Self>,
    files: RefCell<Vec<OptRc<CpioOldLe_File>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for CpioOldLe {
    type Root = CpioOldLe;
    type Parent = CpioOldLe;

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
        *self_rc.files.borrow_mut() = Vec::new();
        {
            let mut _i = 0;
            while !_io.is_eof() {
                let t = Self::read_into::<_, CpioOldLe_File>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.files.borrow_mut().push(t);
                _i += 1;
            }
        }
        Ok(())
    }
}
impl CpioOldLe {
}
impl CpioOldLe {
    pub fn files(&self) -> Ref<'_, Vec<OptRc<CpioOldLe_File>>> {
        self.files.borrow()
    }
}
impl CpioOldLe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct CpioOldLe_File {
    pub(crate) _root: SharedType<CpioOldLe>,
    pub(crate) _parent: SharedType<CpioOldLe>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<CpioOldLe_FileHeader>>,
    path_name: RefCell<Vec<u8>>,
    string_terminator: RefCell<Vec<u8>>,
    path_name_padding: RefCell<Vec<u8>>,
    file_data: RefCell<Vec<u8>>,
    file_data_padding: RefCell<Vec<u8>>,
    end_of_file_padding: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for CpioOldLe_File {
    type Root = CpioOldLe;
    type Parent = CpioOldLe;

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
        let t = Self::read_into::<_, CpioOldLe_FileHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.path_name.borrow_mut() = _io.read_bytes(usize::try_from((((*self_rc.header().path_name_size()) as i32) - ((1) as i32)))?)?;
        *self_rc.string_terminator.borrow_mut() = _io.read_bytes(usize::try_from(1)?)?;
        if !(*self_rc.string_terminator() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file/seq/2".to_string() }));
        }
        if (((*self_rc.header().path_name_size()) as i32) % ((2) as i32)) == 1 {
            *self_rc.path_name_padding.borrow_mut() = _io.read_bytes(usize::try_from(1)?)?;
            if !(*self_rc.path_name_padding() == vec![0x0u8]) {
                return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file/seq/3".to_string() }));
            }
        }
        *self_rc.file_data.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.header().file_size().value()?)?)?;
        if modulo(*self_rc.header().file_size().value()? as i64, 2 as i64) == 1 {
            *self_rc.file_data_padding.borrow_mut() = _io.read_bytes(usize::try_from(1)?)?;
            if !(*self_rc.file_data_padding() == vec![0x0u8]) {
                return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file/seq/5".to_string() }));
            }
        }
        if  ((*self_rc.path_name() == vec![0x54u8, 0x52u8, 0x41u8, 0x49u8, 0x4cu8, 0x45u8, 0x52u8, 0x21u8, 0x21u8, 0x21u8]) && (*self_rc.header().file_size().value()? == 0))  {
            *self_rc.end_of_file_padding.borrow_mut() = _io.read_bytes_full()?;
        }
        Ok(())
    }
}
impl CpioOldLe_File {
}
impl CpioOldLe_File {
    pub fn header(&self) -> Ref<'_, OptRc<CpioOldLe_FileHeader>> {
        self.header.borrow()
    }
}
impl CpioOldLe_File {
    pub fn path_name(&self) -> Ref<'_, Vec<u8>> {
        self.path_name.borrow()
    }
}
impl CpioOldLe_File {
    pub fn string_terminator(&self) -> Ref<'_, Vec<u8>> {
        self.string_terminator.borrow()
    }
}
impl CpioOldLe_File {
    pub fn path_name_padding(&self) -> Ref<'_, Vec<u8>> {
        self.path_name_padding.borrow()
    }
}
impl CpioOldLe_File {
    pub fn file_data(&self) -> Ref<'_, Vec<u8>> {
        self.file_data.borrow()
    }
}
impl CpioOldLe_File {
    pub fn file_data_padding(&self) -> Ref<'_, Vec<u8>> {
        self.file_data_padding.borrow()
    }
}
impl CpioOldLe_File {
    pub fn end_of_file_padding(&self) -> Ref<'_, Vec<u8>> {
        self.end_of_file_padding.borrow()
    }
}
impl CpioOldLe_File {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct CpioOldLe_FileHeader {
    pub(crate) _root: SharedType<CpioOldLe>,
    pub(crate) _parent: SharedType<CpioOldLe_File>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    device_number: RefCell<u16>,
    inode_number: RefCell<u16>,
    mode: RefCell<u16>,
    user_id: RefCell<u16>,
    group_id: RefCell<u16>,
    number_of_links: RefCell<u16>,
    r_device_number: RefCell<u16>,
    modification_time: RefCell<OptRc<CpioOldLe_FourByteUnsignedInteger>>,
    path_name_size: RefCell<u16>,
    file_size: RefCell<OptRc<CpioOldLe_FourByteUnsignedInteger>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for CpioOldLe_FileHeader {
    type Root = CpioOldLe;
    type Parent = CpioOldLe_File;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(usize::try_from(2)?)?;
        if !(*self_rc.magic() == vec![0xc7u8, 0x71u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/0".to_string() }));
        }
        *self_rc.device_number.borrow_mut() = _io.read_u2le()?;
        *self_rc.inode_number.borrow_mut() = _io.read_u2le()?;
        *self_rc.mode.borrow_mut() = _io.read_u2le()?;
        *self_rc.user_id.borrow_mut() = _io.read_u2le()?;
        *self_rc.group_id.borrow_mut() = _io.read_u2le()?;
        *self_rc.number_of_links.borrow_mut() = _io.read_u2le()?;
        *self_rc.r_device_number.borrow_mut() = _io.read_u2le()?;
        let t = Self::read_into::<_, CpioOldLe_FourByteUnsignedInteger>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.modification_time.borrow_mut() = t;
        *self_rc.path_name_size.borrow_mut() = _io.read_u2le()?;
        let t = Self::read_into::<_, CpioOldLe_FourByteUnsignedInteger>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.file_size.borrow_mut() = t;
        Ok(())
    }
}
impl CpioOldLe_FileHeader {
}
impl CpioOldLe_FileHeader {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn device_number(&self) -> Ref<'_, u16> {
        self.device_number.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn inode_number(&self) -> Ref<'_, u16> {
        self.inode_number.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn mode(&self) -> Ref<'_, u16> {
        self.mode.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn user_id(&self) -> Ref<'_, u16> {
        self.user_id.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn group_id(&self) -> Ref<'_, u16> {
        self.group_id.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn number_of_links(&self) -> Ref<'_, u16> {
        self.number_of_links.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn r_device_number(&self) -> Ref<'_, u16> {
        self.r_device_number.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn modification_time(&self) -> Ref<'_, OptRc<CpioOldLe_FourByteUnsignedInteger>> {
        self.modification_time.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn path_name_size(&self) -> Ref<'_, u16> {
        self.path_name_size.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn file_size(&self) -> Ref<'_, OptRc<CpioOldLe_FourByteUnsignedInteger>> {
        self.file_size.borrow()
    }
}
impl CpioOldLe_FileHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct CpioOldLe_FourByteUnsignedInteger {
    pub(crate) _root: SharedType<CpioOldLe>,
    pub(crate) _parent: SharedType<CpioOldLe_FileHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    most_significant_bits: RefCell<u16>,
    least_significant_bits: RefCell<u16>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<i32>,
}
impl KStruct for CpioOldLe_FourByteUnsignedInteger {
    type Root = CpioOldLe;
    type Parent = CpioOldLe_FileHeader;

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
        *self_rc.most_significant_bits.borrow_mut() = _io.read_u2le()?;
        *self_rc.least_significant_bits.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl CpioOldLe_FourByteUnsignedInteger {
    pub fn value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = ((((*self.least_significant_bits()) as i32) + (((((*self.most_significant_bits()) as i32) << ((16) as i32))) as i32))).try_into()?;
        Ok(self.value.borrow())
    }
}
impl CpioOldLe_FourByteUnsignedInteger {
    pub fn most_significant_bits(&self) -> Ref<'_, u16> {
        self.most_significant_bits.borrow()
    }
}
impl CpioOldLe_FourByteUnsignedInteger {
    pub fn least_significant_bits(&self) -> Ref<'_, u16> {
        self.least_significant_bits.borrow()
    }
}
impl CpioOldLe_FourByteUnsignedInteger {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
