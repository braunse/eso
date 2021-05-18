// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! [![Crates.io](https://img.shields.io/crates/v/eso)](https://crates.io/crates/eso)
//! [![docs.rs](https://img.shields.io/docsrs/eso)](https://docs.rs/eso)
//! [![GitHub issues](https://img.shields.io/github/issues/braunse/eso)](https://github.com/braunse/eso/issues)
//! [![GitHub pull requests](https://img.shields.io/github/issues-pr/braunse/eso)](https://github.com/braunse/eso/pulls)
//! ![GitHub last commit](https://img.shields.io/github/last-commit/braunse/eso)
//! ![Crates.io](https://img.shields.io/crates/l/eso)
//!
//! Type-level machinery for building [`Cow`](std::borrow::Cow)-like
//! types while avoiding unnecessary copies of `'static' or
//! other shareable references.
//!
//! The main feature of this crate is the [`Eso`] type, which tracks
//! whether the contained value is ephemeral (i.e. borrowed with any
//! lifetime), static/shared (i.e. can be held on to indefinitely) or
//! owned (i.e. can be moved and may be mutably accessed).
//!
//! In addition, it also statically tracks which of these is *possible*
//! at any given point in the code by encoding the information on a
//! type level using the definitions in the [`maybe`] module.
//!
//! While [`Eso`] is perfectly happy working with normal Rust references,
//! it also provides an abstraction to support a more generalized notion
//! of reference. The definitions in the [`borrow`] module describe
//! the different operations that are required to use generalized
//! references.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[cfg_attr(feature = "unstable-doc-cfg", feature(doc_cfg))]
pub mod borrow;
pub mod eso;
pub mod maybe;
pub mod unify;

#[doc(inline)]
pub use crate::eso::Eso;
pub use crate::eso::{t, x};
#[doc(inline)]
pub use crate::maybe::{An, No};
