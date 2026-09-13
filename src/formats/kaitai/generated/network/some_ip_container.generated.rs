// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::some_ip::SomeIp;

#[derive(Default, Debug, Clone)]
pub struct SomeIpContainer {
    pub(crate) _root: SharedType<SomeIpContainer>,
    pub(crate) _parent: SharedType<SomeIpContainer>,
    pub(crate) _self_shared: SharedType<Self>,
    some_ip_packages: RefCell<Vec<OptRc<SomeIp>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SomeIpContainer {
    type Root = SomeIpContainer;
    type Parent = SomeIpContainer;

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
        *self_rc.some_ip_packages.borrow_mut() = Vec::new();
        {
            let mut _i = 0;
            while !_io.is_eof() {
                let t = Self::read_into::<_, SomeIp>(&*_io, None, None)?.into();
                self_rc.some_ip_packages.borrow_mut().push(t);
                _i += 1;
            }
        }
        Ok(())
    }
}
impl SomeIpContainer {
}
impl SomeIpContainer {
    pub fn some_ip_packages(&self) -> Ref<'_, Vec<OptRc<SomeIp>>> {
        self.some_ip_packages.borrow()
    }
}
impl SomeIpContainer {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
