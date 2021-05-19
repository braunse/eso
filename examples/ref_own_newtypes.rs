// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(dead_code)]

use eso::{
    borrow::{Borrowable, Ownable},
    An, Eso, No,
};
use std::ops::{Deref, DerefMut};

pub struct OwnedString(Eso<No<&'static str>, No<&'static str>, An<String>>);

pub struct StaticString(Eso<No<&'static str>, An<&'static str>, An<String>>);

pub struct StringRef<'a>(Eso<An<&'a str>, An<&'static str>, No<String>>);

impl OwnedString {
    pub fn new(s: String) -> Self {
        Self(Eso::from_owned(s))
    }
}

impl StaticString {
    pub fn new_string(s: String) -> Self {
        Self(Eso::from_owned(s))
    }

    pub const fn new_static(s: &'static str) -> Self {
        Self(Eso::from_static(s))
    }
}

impl<'a> Borrowable<'a, StringRef<'a>> for OwnedString {
    fn borrow(&'a self) -> StringRef<'a> {
        StringRef(self.0.reference())
    }
}

impl<'a> Borrowable<'a, StringRef<'a>> for StaticString {
    fn borrow(&'a self) -> StringRef<'a> {
        StringRef(self.0.reference())
    }
}

impl<'a> Ownable<OwnedString> for StringRef<'a> {
    fn own(&self) -> OwnedString {
        OwnedString(self.0.to_owning().relax())
    }
}

impl<'a> Deref for StringRef<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl Deref for StaticString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl Deref for OwnedString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.0.get_owned_ref()
    }
}

impl DerefMut for OwnedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut()
    }
}
