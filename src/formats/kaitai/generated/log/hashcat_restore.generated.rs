// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <https://hashcat.net/wiki/doku.php?id=restore> Source
 */

#[derive(Default, Debug, Clone)]
pub struct HashcatRestore {
    pub(crate) _root: SharedType<HashcatRestore>,
    pub(crate) _parent: SharedType<HashcatRestore>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u32>,
    cwd: RefCell<String>,
    dicts_pos: RefCell<u32>,
    masks_pos: RefCell<u32>,
    padding: RefCell<Vec<u8>>,
    current_restore_point: RefCell<u64>,
    argc: RefCell<u32>,
    padding2: RefCell<Vec<u8>>,
    argv: RefCell<Vec<String>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for HashcatRestore {
    type Root = HashcatRestore;
    type Parent = HashcatRestore;

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
        *self_rc.version.borrow_mut() = _io.read_u4le()?;
        *self_rc.cwd.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(256_usize)?, 0, false), "UTF-8")?;
        *self_rc.dicts_pos.borrow_mut() = _io.read_u4le()?;
        *self_rc.masks_pos.borrow_mut() = _io.read_u4le()?;
        *self_rc.padding.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc.current_restore_point.borrow_mut() = _io.read_u8le()?;
        *self_rc.argc.borrow_mut() = _io.read_u4le()?;
        *self_rc.padding2.borrow_mut() = _io.read_bytes(12_usize)?;
        *self_rc.argv.borrow_mut() = Vec::new();
        let l_argv = usize::try_from(*self_rc.argc())?;
        for _i in 0_usize..l_argv {
            self_rc.argv.borrow_mut().push(bytes_to_str(&_io.read_bytes_term(10, false, true, true)?, "UTF-8")?);
        }
        Ok(())
    }
}
impl HashcatRestore {
}
impl HashcatRestore {
    pub fn version(&self) -> Ref<'_, u32> {
        self.version.borrow()
    }
}
impl HashcatRestore {
    pub fn cwd(&self) -> Ref<'_, String> {
        self.cwd.borrow()
    }
}
impl HashcatRestore {
    pub fn dicts_pos(&self) -> Ref<'_, u32> {
        self.dicts_pos.borrow()
    }
}
impl HashcatRestore {
    pub fn masks_pos(&self) -> Ref<'_, u32> {
        self.masks_pos.borrow()
    }
}
impl HashcatRestore {
    pub fn padding(&self) -> Ref<'_, Vec<u8>> {
        self.padding.borrow()
    }
}
impl HashcatRestore {
    pub fn current_restore_point(&self) -> Ref<'_, u64> {
        self.current_restore_point.borrow()
    }
}
impl HashcatRestore {
    pub fn argc(&self) -> Ref<'_, u32> {
        self.argc.borrow()
    }
}
impl HashcatRestore {
    pub fn padding2(&self) -> Ref<'_, Vec<u8>> {
        self.padding2.borrow()
    }
}
impl HashcatRestore {
    pub fn argv(&self) -> Ref<'_, Vec<String>> {
        self.argv.borrow()
    }
}
impl HashcatRestore {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
