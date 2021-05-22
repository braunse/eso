// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The [`Take`] and [`Borrow`] traits abstract over the duality
//! between owned and borrowed types, much in the same way as the
//! standard-library [`Borrow`](std::borrow::Borrow) and
//! [`ToOwned`](std::borrow::ToOwned) traits do.
//!
//! The difference between these and the standard-library traits is
//! that the traits here are more generic. See the specific traits
//! for details.
//!
//! `Eso` differentiates three different categories of values:
//!
//! |       | Category  | Analogy            | Description
//! |------:|-----------|--------------------|----------------------
//! | **E** | Ephemeral | `&'a `[`str`]      | A reference to a shared value with a limited lifetime
//! | **S** | Static    | `&'static `[`str`] | A reference to a shared value that client code can hold on to indefinitely
//! | **O** | Owned     | [`String`]         | A value that is exclusively owned and can be mutated (if no references to it have been borrowed out)
//!
//! These characteristics are enforced by [`Eso`](crate::eso::Eso)
//! only on a per-function basis, but not in general. Client code should
//! enforce them where necessary.
//!
//! This module defines traits to convert between the categories:
//!
//! | From      | To Ephemeral | To Static                                          | To Owned
//! |-----------|--------------|----------------------------------------------------|----------
//! | Ephemeral |              | `TryInternRef`, `InternRef`                        | [`Take`]
//! | Static    | [`Borrow`]   |                                                    | [`Take`]
//! | Owned     | [`Borrow`]   | `TryInternRef`, `TryIntern`, `InternRef`, `Intern` |
//!
//! As can be seen from the table, there is some additional complexity
//! regarding the interning operation:
//!
//!  1. **Interning may fail:** Depending on the implementation, not all
//!     values may have a static counterpart.
//!  2. **Owned values may offer optimization opportunities:** If the
//!     owned value is not needed after the interning operation, it is
//!     cheaper to move it into the interning function.
//!
//! ## Open questions / TODO
//!
//!  - [ ] actually implement the `...Intern...` traits
//!  - [ ] think about naming:
//!    - `Borrow` clashes with `std`
//!    - `Take` does not seem like a good description of what is actually
//!      happening
//!  - [ ] is `Borrow`ing from an Owned really the same operation as
//!    `Borrow`ing from a static reference?
use std::{
    borrow::Cow,
    ffi::{CStr, CString, OsStr, OsString},
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

/// A value that can be borrowed as a generalized reference of type `T`.
///
/// ```
/// # use eso::borrow::Borrow;
/// let value = String::from("Hello World");
/// let reference: &str = value.borrow();
/// ```
///
/// The difference to [`Borrow`](std::borrow::Borrow) is that
/// this trait allows you to return types that are not actually references,
/// such as [`Cow`]s:
///
/// ```
/// # use eso::borrow::Borrow; use std::borrow::Cow;
/// let value = String::from("Hello World");
/// let reference: Cow<str> = value.borrow();
/// ```
pub trait Borrow<'a, T: 'a> {
    /// Borrow a generalized reference of type `T`.
    fn borrow(&'a self) -> T;
}

impl<'a, T: ?Sized> Borrow<'a, &'a T> for Box<T> {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        &**self
    }
}

impl<'a, T: ?Sized> Borrow<'a, &'a T> for Arc<T> {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        &**self
    }
}

impl<'a, T: ?Sized> Borrow<'a, &'a T> for Rc<T> {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        &**self
    }
}

impl<'a, T> Borrow<'a, &'a T> for T {
    fn borrow(&'a self) -> &'a T {
        self
    }
}

impl<'a, T, R> Borrow<'a, Cow<'a, R>> for T
where
    T: std::borrow::Borrow<R>,
    R: ?Sized + ToOwned<Owned = T>,
{
    fn borrow(&'a self) -> Cow<R> {
        Cow::Borrowed(self.borrow())
    }
}

impl<'a> Borrow<'a, &'a str> for String {
    #[inline]
    fn borrow(&'a self) -> &'a str {
        self.as_str()
    }
}

impl<'a> Borrow<'a, &'a [u8]> for String {
    #[inline]
    fn borrow(&'a self) -> &'a [u8] {
        self.as_bytes()
    }
}

impl<'a> Borrow<'a, &'a Path> for PathBuf {
    #[inline]
    fn borrow(&self) -> &Path {
        self.as_path()
    }
}

impl<'a, T> Borrow<'a, &'a [T]> for Vec<T> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a> Borrow<'a, &'a OsStr> for OsString {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<'a> Borrow<'a, &'a OsStr> for PathBuf {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<'a> Borrow<'a, &'a CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr {
        self.as_c_str()
    }
}

impl<'a, 'b: 'a, T: ?Sized> Borrow<'a, &'a T> for &'b T {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        *self
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    impl<'a> Borrow<'a, &'a [u8]> for OsString {
        #[inline]
        fn borrow(&self) -> &[u8] {
            self.as_os_str().as_bytes()
        }
    }

    impl<'a> Borrow<'a, &'a [u8]> for PathBuf {
        #[inline]
        fn borrow(&self) -> &[u8] {
            self.as_os_str().as_bytes()
        }
    }

    impl<'a> Take<OsString> for &'a [u8] {
        fn to_owned(&self) -> OsString {
            OsStr::from_bytes(self).to_os_string()
        }
    }

    impl<'a> Take<PathBuf> for &'a [u8] {
        fn to_owned(&self) -> PathBuf {
            PathBuf::from(OsStr::from_bytes(self))
        }
    }
}

/// A version of the [`ToOwned`] trait describing *generalized* references
/// from which an owned form can be cloned.
///
/// The obvious instances for [`str`], [`Path`], [`OsStr`], [`CStr`],
/// slices as well as sized types implementing [`Clone`] are implemented
/// out of the box.
pub trait Take<O>: Sized {
    /// Clone the thing denoted by a generalized reference into one that
    /// is owned.
    ///
    /// This trait function defaults to calling [`to_owned`](Take::to_owned)
    /// but is there as an optimization opportunity, if needed.
    fn own(self) -> O {
        self.to_owned()
    }

    /// Clone the thing denoted by a generalized reference into one that
    /// is owned without consuming the reference.
    fn to_owned(&self) -> O;
}

impl<'a> Take<String> for &'a str {
    fn to_owned(&self) -> String {
        self.to_string()
    }
}

impl<'a> Take<PathBuf> for &'a Path {
    fn to_owned(&self) -> PathBuf {
        self.to_path_buf()
    }
}

impl<'a, T: Clone> Take<Vec<T>> for &'a [T] {
    fn to_owned(&self) -> Vec<T> {
        self.to_vec()
    }
}

impl<'a> Take<OsString> for &'a OsStr {
    fn to_owned(&self) -> OsString {
        self.to_os_string()
    }
}

impl<'a> Take<PathBuf> for &'a OsStr {
    fn to_owned(&self) -> PathBuf {
        PathBuf::from(self)
    }
}

impl<'a, T: Clone> Take<T> for &'a T {
    fn to_owned(&self) -> T {
        (*self).clone()
    }
}

impl<'a> Take<CString> for &'a CStr {
    fn to_owned(&self) -> CString {
        (*self).to_owned()
    }
}

impl<'a, T: Clone> Take<Box<T>> for &'a T {
    fn to_owned(&self) -> Box<T> {
        Box::new((*self).clone())
    }
}

impl<'a, T: Clone> Take<Rc<T>> for &'a T {
    fn to_owned(&self) -> Rc<T> {
        Rc::new((*self).clone())
    }
}

impl<'a, T: Clone> Take<Arc<T>> for &'a T {
    fn to_owned(&self) -> Arc<T> {
        Arc::new((*self).clone())
    }
}

impl<'a, R: ToOwned> Take<R::Owned> for Cow<'a, R> {
    fn to_owned(&self) -> R::Owned {
        self.clone().into_owned()
    }
}
