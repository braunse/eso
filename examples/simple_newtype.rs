// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(dead_code)]

use std::{borrow::Cow, ops::Deref};

use eso::{
    borrow::{Borrowable, Ownable},
    An, Eso,
};

#[derive(Debug, Clone)]
pub struct WrappedString<'a>(Eso<An<&'a str>, An<&'static str>, An<String>>);

impl<'a> WrappedString<'a> {
    pub const fn from_str(s: &'a str) -> Self {
        WrappedString(Eso::from_ref(s))
    }

    pub const fn from_static(s: &'static str) -> Self {
        WrappedString(Eso::from_static(s))
    }

    pub const fn from_string(s: String) -> Self {
        WrappedString(Eso::from_owned(s))
    }

    pub fn from_cow<'b>(s: Cow<'b, str>) -> Self
    where
        'b: 'a,
    {
        WrappedString(Eso::from_cow(s))
    }

    pub fn narrow<'b>(&self) -> WrappedString<'b>
    where
        'a: 'b,
    {
        WrappedString(self.0.clone_relax())
    }

    pub fn into_static(self) -> WrappedString<'static> {
        WrappedString(self.0.into_static().relax())
    }

    pub fn into_owning(self) -> WrappedString<'static> {
        WrappedString(self.0.into_owning().relax())
    }
}

impl<'a, 'b: 'a> From<&'b str> for WrappedString<'a> {
    fn from(s: &'b str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for WrappedString<'_> {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl<'a, 'b: 'a> From<Cow<'b, str>> for WrappedString<'a> {
    fn from(s: Cow<'b, str>) -> Self {
        WrappedString(Eso::from_cow(s))
    }
}

impl AsRef<str> for WrappedString<'_> {
    fn as_ref(&self) -> &str {
        self.0.get_ref()
    }
}

impl<'a, 'b: 'a> Borrowable<'a, WrappedString<'a>> for WrappedString<'b> {
    fn borrow(&'a self) -> WrappedString<'a> {
        self.narrow()
    }
}

impl<'a> Ownable<WrappedString<'static>> for WrappedString<'a> {
    fn own(&self) -> WrappedString<'static> {
        WrappedString(self.0.clone().into_owning().relax())
    }
}

impl<'a> Deref for WrappedString<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.get_ref()
    }
}
