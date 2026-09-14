// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Microsoft Windows SYSTEMTIME structure, stores individual components
 * of date and time as individual fields, up to millisecond precision.
 * \sa <https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime> Source
 */

#[derive(Default, Debug, Clone)]
pub struct WindowsSystemtime {
    pub(crate) _root: SharedType<WindowsSystemtime>,
    pub(crate) _parent: SharedType<WindowsSystemtime>,
    pub(crate) _self_shared: SharedType<Self>,
    year: RefCell<u16>,
    month: RefCell<u16>,
    dow: RefCell<u16>,
    day: RefCell<u16>,
    hour: RefCell<u16>,
    min: RefCell<u16>,
    sec: RefCell<u16>,
    msec: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for WindowsSystemtime {
    type Root = WindowsSystemtime;
    type Parent = WindowsSystemtime;

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
        *self_rc.year.borrow_mut() = _io.read_u2le()?;
        *self_rc.month.borrow_mut() = _io.read_u2le()?;
        *self_rc.dow.borrow_mut() = _io.read_u2le()?;
        *self_rc.day.borrow_mut() = _io.read_u2le()?;
        *self_rc.hour.borrow_mut() = _io.read_u2le()?;
        *self_rc.min.borrow_mut() = _io.read_u2le()?;
        *self_rc.sec.borrow_mut() = _io.read_u2le()?;
        *self_rc.msec.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl WindowsSystemtime {
}

/**
 * Year
 */
impl WindowsSystemtime {
    pub fn year(&self) -> Ref<'_, u16> {
        self.year.borrow()
    }
}

/**
 * Month (January = 1)
 */
impl WindowsSystemtime {
    pub fn month(&self) -> Ref<'_, u16> {
        self.month.borrow()
    }
}

/**
 * Day of week (Sun = 0)
 */
impl WindowsSystemtime {
    pub fn dow(&self) -> Ref<'_, u16> {
        self.dow.borrow()
    }
}

/**
 * Day of month
 */
impl WindowsSystemtime {
    pub fn day(&self) -> Ref<'_, u16> {
        self.day.borrow()
    }
}

/**
 * Hours
 */
impl WindowsSystemtime {
    pub fn hour(&self) -> Ref<'_, u16> {
        self.hour.borrow()
    }
}

/**
 * Minutes
 */
impl WindowsSystemtime {
    pub fn min(&self) -> Ref<'_, u16> {
        self.min.borrow()
    }
}

/**
 * Seconds
 */
impl WindowsSystemtime {
    pub fn sec(&self) -> Ref<'_, u16> {
        self.sec.borrow()
    }
}

/**
 * Milliseconds
 */
impl WindowsSystemtime {
    pub fn msec(&self) -> Ref<'_, u16> {
        self.msec.borrow()
    }
}
impl WindowsSystemtime {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
