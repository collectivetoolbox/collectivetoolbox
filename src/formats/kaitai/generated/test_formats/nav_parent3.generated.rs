// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/*
== License information for parts derived from kaitai_struct_tests, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

MIT License

Copyright (c) 2019 Kaitai Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


*/
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct NavParent3 {
    pub(crate) _root: SharedType<NavParent3>,
    pub(crate) _parent: SharedType<NavParent3>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs_tags: RefCell<u32>,
    num_tags: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_tags: Cell<bool>,
    tags: RefCell<Vec<OptRc<NavParent3_Tag>>>,
}
impl KStruct for NavParent3 {
    type Root = NavParent3;
    type Parent = NavParent3;

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
        *self_rc.ofs_tags.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_tags.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParent3 {
    pub fn tags(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<NavParent3_Tag>>>> {
        let _io = self._io.borrow();
        if self.f_tags.get() {
            return Ok(self.tags.borrow());
        }
        self.f_tags.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs_tags())?)?;
        *self.tags.borrow_mut() = Vec::new();
        let l_tags = usize::try_from(*self.num_tags())?;
        for _i in 0_usize..l_tags {
            let t = Self::read_into::<_, NavParent3_Tag>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
            self.tags.borrow_mut().push(t);
        }
        _io.seek(_pos)?;
        Ok(self.tags.borrow())
    }
}
impl NavParent3 {
    pub fn ofs_tags(&self) -> Ref<'_, u32> {
        self.ofs_tags.borrow()
    }
}
impl NavParent3 {
    pub fn num_tags(&self) -> Ref<'_, u32> {
        self.num_tags.borrow()
    }
}
impl NavParent3 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParent3_Tag {
    pub(crate) _root: SharedType<NavParent3>,
    pub(crate) _parent: SharedType<NavParent3>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    ofs: RefCell<u32>,
    num_items: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_tag_content: Cell<bool>,
    tag_content: RefCell<Option<NavParent3_Tag_TagContent>>,
}
#[derive(Debug, Clone)]
pub enum NavParent3_Tag_TagContent {
    NavParent3_Tag_TagChar(OptRc<NavParent3_Tag_TagChar>),
}
impl From<&NavParent3_Tag_TagContent> for OptRc<NavParent3_Tag_TagChar> {
    fn from(v: &NavParent3_Tag_TagContent) -> Self {
        let NavParent3_Tag_TagContent::NavParent3_Tag_TagChar(x) = v;
        x.clone()
    }
}
impl TryFrom<&NavParent3_Tag_TagContent> for OptRc<NavParent3_Tag_TagChar> {
    type Error = KError;
    fn try_from(v: &NavParent3_Tag_TagContent) -> Result<Self, Self::Error> {
        if let NavParent3_Tag_TagContent::NavParent3_Tag_TagChar(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<NavParent3_Tag_TagChar>> for NavParent3_Tag_TagContent {
    fn from(v: OptRc<NavParent3_Tag_TagChar>) -> Self {
        Self::NavParent3_Tag_TagChar(v)
    }
}
impl KStruct for NavParent3_Tag {
    type Root = NavParent3;
    type Parent = NavParent3;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        *self_rc.ofs.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_items.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParent3_Tag {
    pub fn tag_content(
        &self
    ) -> KResult<Ref<'_, Option<NavParent3_Tag_TagContent>>> {
        let _io = self._io.borrow();
        if self.f_tag_content.get() {
            return Ok(self.tag_content.borrow());
        }
        self.f_tag_content.set(true);
        let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
        let _pos = io.pos();
        io.seek(usize::try_from(*self.ofs())?)?;
        match self.name().as_str() {
            "RAHC" => {
                let t = Self::read_into::<_, NavParent3_Tag_TagChar>(&io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.tag_content.borrow_mut() = Some(t);
            }
            _ => {}
        }
        io.seek(_pos)?;
        Ok(self.tag_content.borrow())
    }
}
impl NavParent3_Tag {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl NavParent3_Tag {
    pub fn ofs(&self) -> Ref<'_, u32> {
        self.ofs.borrow()
    }
}
impl NavParent3_Tag {
    pub fn num_items(&self) -> Ref<'_, u32> {
        self.num_items.borrow()
    }
}
impl NavParent3_Tag {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParent3_Tag_TagChar {
    pub(crate) _root: SharedType<NavParent3>,
    pub(crate) _parent: SharedType<NavParent3_Tag>,
    pub(crate) _self_shared: SharedType<Self>,
    content: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParent3_Tag_TagChar {
    type Root = NavParent3;
    type Parent = NavParent3_Tag;

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
        *self_rc.content.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.num_items())?)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParent3_Tag_TagChar {
}
impl NavParent3_Tag_TagChar {
    pub fn content(&self) -> Ref<'_, String> {
        self.content.borrow()
    }
}
impl NavParent3_Tag_TagChar {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
