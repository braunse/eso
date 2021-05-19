// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The [`Ownable`] and [`Borrowable`] traits abstract over the duality
//! between owned and borrowed types, much in the same way as the
//! standard-library [`Borrow`](std::borrow::Borrow) and
//! [`ToOwned`](std::borrow::ToOwned) traits do.
//!
//! The difference between these and the standard-library traits is
//! that the traits here are more generic. See the specific traits
//! for details.

use std::{
    borrow::{Borrow, Cow},
    ffi::{CStr, CString, OsStr, OsString},
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

/// A value that can be borrowed as a generalized reference of type `T`.
///
/// ```
/// # use eso::borrow::Borrowable;
/// let value = String::from("Hello World");
/// let reference: &str = value.borrow();
/// ```
///
/// The difference to [`Borrow`](std::borrow::Borrow) is that
/// this trait allows you to return types that are not actually references,
/// such as [`Cow`]s:
///
/// ```
/// # use eso::borrow::Borrowable; use std::borrow::Cow;
/// let value = String::from("Hello World");
/// let reference: Cow<str> = value.borrow();
/// ```
pub trait Borrowable<'a, T: 'a> {
    /// Borrow a generalized reference of type `T`.
    fn borrow(&'a self) -> T;
}

impl<'a, T: ?Sized> Borrowable<'a, &'a T> for Box<T> {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        &**self
    }
}

impl<'a, T: ?Sized> Borrowable<'a, &'a T> for Arc<T> {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        &**self
    }
}

impl<'a, T: ?Sized> Borrowable<'a, &'a T> for Rc<T> {
    #[inline]
    fn borrow(&'a self) -> &'a T {
        &**self
    }
}

impl<'a, T> Borrowable<'a, &'a T> for T {
    fn borrow(&'a self) -> &'a T {
        self
    }
}

impl<'a, T, R> Borrowable<'a, Cow<'a, R>> for T
where
    T: Borrow<R>,
    R: ?Sized + ToOwned<Owned = T>,
{
    fn borrow(&'a self) -> Cow<R> {
        Cow::Borrowed(self.borrow())
    }
}

impl<'a> Borrowable<'a, &'a str> for String {
    #[inline]
    fn borrow(&'a self) -> &'a str {
        self.as_str()
    }
}

impl<'a> Borrowable<'a, &'a [u8]> for String {
    #[inline]
    fn borrow(&'a self) -> &'a [u8] {
        self.as_bytes()
    }
}

impl<'a> Borrowable<'a, &'a Path> for PathBuf {
    #[inline]
    fn borrow(&self) -> &Path {
        self.as_path()
    }
}

impl<'a, T> Borrowable<'a, &'a [T]> for Vec<T> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a> Borrowable<'a, &'a OsStr> for OsString {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<'a> Borrowable<'a, &'a OsStr> for PathBuf {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<'a> Borrowable<'a, &'a CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr {
        self.as_c_str()
    }
}

/// Conversion from one generalized reference type into another.
///
/// This trait is equivalent to [`Borrowable`], but from the starting
/// point of a given generalized reference instead of from the object
/// that should be denoted by the resulting value.
///
/// The canonical example is the subtyping relationship between
/// standard Rust references: `&'a T` is convertible to `&'b T` if
/// `'a` outlives `'b'.
///
/// Since we abstract over references, we cannot rely on the automatic
/// support for subtyping in the compiler anymore and have to make it
/// explicit using this trait.
///
/// The canonical example above translates to the following instance
/// (which is already implemented in this crate):
///
/// ```ignore
/// impl<'a: 'b, 'b, T: ?Sized> Reborrowable<'b, &'b T> for &'a T {
///     fn reborrow(self) -> &'b T {
///         // subtyping works once the compiler statically knows
///         // these are references:
///         self
///     }
/// }
/// ```
pub trait Reborrowable<'a, T: 'a> {
    /// Convert a generalized reference of type `Self` into a
    /// generalized reference of type `T`
    fn reborrow(self) -> T;
}

impl<'a: 'b, 'b, T: ?Sized> Reborrowable<'b, &'b T> for &'a T {
    fn reborrow(self) -> &'b T {
        self
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    impl<'a> Borrowable<'a, &'a [u8]> for OsString {
        #[inline]
        fn borrow(&self) -> &[u8] {
            self.as_os_str().as_bytes()
        }
    }

    impl<'a> Borrowable<'a, &'a [u8]> for PathBuf {
        #[inline]
        fn borrow(&self) -> &[u8] {
            self.as_os_str().as_bytes()
        }
    }

    impl<'a> Ownable<OsString> for &'a [u8] {
        fn own(&self) -> OsString {
            OsStr::from_bytes(self).to_os_string()
        }
    }

    impl<'a> Ownable<PathBuf> for &'a [u8] {
        fn own(&self) -> PathBuf {
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
pub trait Ownable<O>: Sized {
    /// Clone the thing denoted by a generalized reference into one that
    /// is owned (does not reference another object)
    fn own(&self) -> O;
}

impl<'a> Ownable<String> for &'a str {
    fn own(&self) -> String {
        self.to_string()
    }
}

impl<'a> Ownable<PathBuf> for &'a Path {
    fn own(&self) -> PathBuf {
        self.to_path_buf()
    }
}

impl<'a, T: Clone> Ownable<Vec<T>> for &'a [T] {
    fn own(&self) -> Vec<T> {
        self.to_vec()
    }
}

impl<'a> Ownable<OsString> for &'a OsStr {
    fn own(&self) -> OsString {
        self.to_os_string()
    }
}

impl<'a> Ownable<PathBuf> for &'a OsStr {
    fn own(&self) -> PathBuf {
        PathBuf::from(self)
    }
}

impl<'a, T: Clone> Ownable<T> for &'a T {
    fn own(&self) -> T {
        (*self).clone()
    }
}

impl<'a> Ownable<CString> for &'a CStr {
    fn own(&self) -> CString {
        (*self).to_owned()
    }
}

impl<'a, T: Clone> Ownable<Box<T>> for &'a T {
    fn own(&self) -> Box<T> {
        Box::new((*self).clone())
    }
}

impl<'a, T: Clone> Ownable<Rc<T>> for &'a T {
    fn own(&self) -> Rc<T> {
        Rc::new((*self).clone())
    }
}

impl<'a, T: Clone> Ownable<Arc<T>> for &'a T {
    fn own(&self) -> Arc<T> {
        Arc::new((*self).clone())
    }
}

impl<'a, R: ToOwned> Ownable<R::Owned> for Cow<'a, R> {
    fn own(&self) -> R::Owned {
        self.clone().into_owned()
    }
}
