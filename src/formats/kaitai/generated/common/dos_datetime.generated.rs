// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * MS-DOS date and time are packed 16-bit values that specify local date/time.
 * The time is always stored in the current UTC time offset set on the computer
 * which created the file. Note that the daylight saving time (DST) shifts
 * also change the UTC time offset.
 *
 * For example, if you pack two files A and B into a ZIP archive, file A last modified
 * at 2020-03-29 00:59 UTC+00:00 (GMT) and file B at 2020-03-29 02:00 UTC+01:00 (BST),
 * the file modification times saved in MS-DOS format in the ZIP file will vary depending
 * on whether the computer packing the files is set to GMT or BST at the time of ZIP creation.
 *
 *   - If set to GMT:
 *       - file A: 2020-03-29 00:59 (UTC+00:00)
 *       - file B: 2020-03-29 01:00 (UTC+00:00)
 *   - If set to BST:
 *       - file A: 2020-03-29 01:59 (UTC+01:00)
 *       - file B: 2020-03-29 02:00 (UTC+01:00)
 *
 * It follows that you are unable to determine the actual last modified time
 * of any file stored in the ZIP archive, if you don't know the locale time
 * setting of the computer at the time it created the ZIP.
 *
 * This format is used in some data formats from the MS-DOS era, for example:
 *
 *   - [zip](/zip/)
 *   - [rar](/rar/)
 *   - [vfat](/vfat/) (FAT12)
 *   - [lzh](/lzh/)
 *   - [cab](http://justsolve.archiveteam.org/wiki/Cabinet)
 * \sa <https://learn.microsoft.com/en-us/windows/win32/sysinfo/ms-dos-date-and-time> Source
 * \sa <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-dosdatetimetofiletime> Source
 * \sa https://github.com/reactos/reactos/blob/c6b64448ce4/dll/win32/kernel32/client/time.c#L82-L87 DosDateTimeToFileTime
 * \sa https://download.microsoft.com/download/0/8/4/084c452b-b772-4fe5-89bb-a0cbf082286a/fatgen103.doc page 25/34
 */

#[derive(Default, Debug, Clone)]
pub struct DosDatetime {
    pub(crate) _root: SharedType<DosDatetime>,
    pub(crate) _parent: SharedType<DosDatetime>,
    pub(crate) _self_shared: SharedType<Self>,
    time: RefCell<OptRc<DosDatetime_Time>>,
    date: RefCell<OptRc<DosDatetime_Date>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DosDatetime {
    type Root = DosDatetime;
    type Parent = DosDatetime;

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
        let t = Self::read_into::<_, DosDatetime_Time>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.time.borrow_mut() = t;
        let t = Self::read_into::<_, DosDatetime_Date>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.date.borrow_mut() = t;
        Ok(())
    }
}
impl DosDatetime {
}
impl DosDatetime {
    pub fn time(&self) -> Ref<'_, OptRc<DosDatetime_Time>> {
        self.time.borrow()
    }
}
impl DosDatetime {
    pub fn date(&self) -> Ref<'_, OptRc<DosDatetime_Date>> {
        self.date.borrow()
    }
}
impl DosDatetime {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DosDatetime_Date {
    pub(crate) _root: SharedType<DosDatetime>,
    pub(crate) _parent: SharedType<DosDatetime>,
    pub(crate) _self_shared: SharedType<Self>,
    day: RefCell<u64>,
    month: RefCell<u64>,
    year_minus_1980: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_padded_day: Cell<bool>,
    padded_day: RefCell<String>,
    f_padded_month: Cell<bool>,
    padded_month: RefCell<String>,
    f_padded_year: Cell<bool>,
    padded_year: RefCell<String>,
    f_year: Cell<bool>,
    year: RefCell<i32>,
}
impl KStruct for DosDatetime_Date {
    type Root = DosDatetime;
    type Parent = DosDatetime;

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
        *self_rc.day.borrow_mut() = _io.read_bits_int_be(5)?;
        let min_val: u64 = (1).try_into()?;
        if !(*self_rc.day() >= min_val) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::LessThan, src_path: "/types/date/seq/0".to_string() }));
        }
        *self_rc.month.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc.year_minus_1980.borrow_mut() = _io.read_bits_int_be(7)?;
        Ok(())
    }
}
impl DosDatetime_Date {
    pub fn padded_day(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_padded_day.get() {
            return Ok(self.padded_day.borrow());
        }
        self.f_padded_day.set(true);
        *self.padded_day.borrow_mut() = format!("{}{}", if *self.day() <= 9 { "0".to_string() } else { "".to_string() }, self.day().to_string()).to_string();
        Ok(self.padded_day.borrow())
    }
    pub fn padded_month(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_padded_month.get() {
            return Ok(self.padded_month.borrow());
        }
        self.f_padded_month.set(true);
        *self.padded_month.borrow_mut() = format!("{}{}", if *self.month() <= 9 { "0".to_string() } else { "".to_string() }, self.month().to_string()).to_string();
        Ok(self.padded_month.borrow())
    }
    pub fn padded_year(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_padded_year.get() {
            return Ok(self.padded_year.borrow());
        }
        self.f_padded_year.set(true);
        *self.padded_year.borrow_mut() = format!("{}{}", if *self.year()? <= 999 { format!("{}{}", "0", if *self.year()? <= 99 { format!("{}{}", "0", if *self.year()? <= 9 { "0".to_string() } else { "".to_string() }).to_string() } else { "".to_string() }).to_string() } else { "".to_string() }, self.year()?.to_string()).to_string();
        Ok(self.padded_year.borrow())
    }

    /**
     * only years from 1980 to 2107 (1980 + 127) can be represented
     */
    pub fn year(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_year.get() {
            return Ok(self.year.borrow());
        }
        self.f_year.set(true);
        *self.year.borrow_mut() = ((1980_u64).saturating_add(*self.year_minus_1980())).try_into()?;
        Ok(self.year.borrow())
    }
}
impl DosDatetime_Date {
    pub fn day(&self) -> Ref<'_, u64> {
        self.day.borrow()
    }
}
impl DosDatetime_Date {
    pub fn month(&self) -> Ref<'_, u64> {
        self.month.borrow()
    }
}
impl DosDatetime_Date {
    pub fn year_minus_1980(&self) -> Ref<'_, u64> {
        self.year_minus_1980.borrow()
    }
}
impl DosDatetime_Date {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DosDatetime_Time {
    pub(crate) _root: SharedType<DosDatetime>,
    pub(crate) _parent: SharedType<DosDatetime>,
    pub(crate) _self_shared: SharedType<Self>,
    second_div_2: RefCell<u64>,
    minute: RefCell<u64>,
    hour: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_padded_hour: Cell<bool>,
    padded_hour: RefCell<String>,
    f_padded_minute: Cell<bool>,
    padded_minute: RefCell<String>,
    f_padded_second: Cell<bool>,
    padded_second: RefCell<String>,
    f_second: Cell<bool>,
    second: RefCell<i32>,
}
impl KStruct for DosDatetime_Time {
    type Root = DosDatetime;
    type Parent = DosDatetime;

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
        *self_rc.second_div_2.borrow_mut() = _io.read_bits_int_be(5)?;
        *self_rc.minute.borrow_mut() = _io.read_bits_int_be(6)?;
        *self_rc.hour.borrow_mut() = _io.read_bits_int_be(5)?;
        Ok(())
    }
}
impl DosDatetime_Time {
    pub fn padded_hour(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_padded_hour.get() {
            return Ok(self.padded_hour.borrow());
        }
        self.f_padded_hour.set(true);
        *self.padded_hour.borrow_mut() = format!("{}{}", if *self.hour() <= 9 { "0".to_string() } else { "".to_string() }, self.hour().to_string()).to_string();
        Ok(self.padded_hour.borrow())
    }
    pub fn padded_minute(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_padded_minute.get() {
            return Ok(self.padded_minute.borrow());
        }
        self.f_padded_minute.set(true);
        *self.padded_minute.borrow_mut() = format!("{}{}", if *self.minute() <= 9 { "0".to_string() } else { "".to_string() }, self.minute().to_string()).to_string();
        Ok(self.padded_minute.borrow())
    }
    pub fn padded_second(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_padded_second.get() {
            return Ok(self.padded_second.borrow());
        }
        self.f_padded_second.set(true);
        *self.padded_second.borrow_mut() = format!("{}{}", if *self.second()? <= 9 { "0".to_string() } else { "".to_string() }, self.second()?.to_string()).to_string();
        Ok(self.padded_second.borrow())
    }
    pub fn second(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_second.get() {
            return Ok(self.second.borrow());
        }
        self.f_second.set(true);
        *self.second.borrow_mut() = ((2_u64).saturating_mul(*self.second_div_2())).try_into()?;
        Ok(self.second.borrow())
    }
}
impl DosDatetime_Time {
    pub fn second_div_2(&self) -> Ref<'_, u64> {
        self.second_div_2.borrow()
    }
}
impl DosDatetime_Time {
    pub fn minute(&self) -> Ref<'_, u64> {
        self.minute.borrow()
    }
}
impl DosDatetime_Time {
    pub fn hour(&self) -> Ref<'_, u64> {
        self.hour.borrow()
    }
}
impl DosDatetime_Time {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
