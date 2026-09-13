// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::{BytesReader, KResult, KStream, KStruct, OptRc, SharedType};
use std::cell::{Ref, RefCell};

/**
 * Microsoft Windows SYSTEMTIME structure, stores individual components
 * of date and time as individual fields, up to millisecond precision.
 * \sa <https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime> Source
 */

#[derive(Default, Debug, Clone)]
pub struct WindowsSystemtime {
    pub(crate) root: SharedType<WindowsSystemtime>,
    pub(crate) parent: SharedType<WindowsSystemtime>,
    pub(crate) self_shared: SharedType<Self>,
    pub year: RefCell<u16>,
    pub month: RefCell<u16>,
    pub dow: RefCell<u16>,
    pub day: RefCell<u16>,
    pub hour: RefCell<u16>,
    pub min: RefCell<u16>,
    pub sec: RefCell<u16>,
    pub msec: RefCell<u16>,
    io: RefCell<BytesReader>,
}
impl KStruct for WindowsSystemtime {
    type Root = WindowsSystemtime;
    type Parent = WindowsSystemtime;

    fn read<S: KStream>(
        self_rc: &OptRc<Self>,
        io: &S,
        root: SharedType<Self::Root>,
        parent: SharedType<Self::Parent>,
    ) -> KResult<()> {
        *self_rc.io.borrow_mut() = io.clone();
        self_rc.root.set(root.get());
        self_rc.parent.set(parent.get());
        self_rc.self_shared.set(Ok(self_rc.clone()));
        *self_rc.year.borrow_mut() = io.read_u2le()?;
        *self_rc.month.borrow_mut() = io.read_u2le()?;
        *self_rc.dow.borrow_mut() = io.read_u2le()?;
        *self_rc.day.borrow_mut() = io.read_u2le()?;
        *self_rc.hour.borrow_mut() = io.read_u2le()?;
        *self_rc.min.borrow_mut() = io.read_u2le()?;
        *self_rc.sec.borrow_mut() = io.read_u2le()?;
        *self_rc.msec.borrow_mut() = io.read_u2le()?;
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
 * Month (1 = January, 2 = February, etc.)
 */
impl WindowsSystemtime {
    pub fn month(&self) -> Ref<'_, u16> {
        self.month.borrow()
    }
}

/**
 * Day of week (0 = Sunday, 1 = Monday, etc.)
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
        self.io.borrow()
    }
}
