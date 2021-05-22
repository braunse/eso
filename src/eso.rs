// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The [`Eso`] type and associated traits for advanced reference
//! wrangling.
//!
//! **Beware** that the flexibility of [`Eso`] comes at a rather
//! high cost in ergonomics.
//! The types are pretty complex and the `where` clauses are
//! unwieldy.
//! You should think twice before exposing anything related to it
//! on the surface of your library.
//!
//! [`Eso`] is meant as a building block for libraries that
//! need the additional flexibility compared to the standard [`Cow`]
//! type.

use crate::shorthand::x;

// You have been warned:
// #![allow(clippy::clippy::type_complexity)]

/// A three-way choice between an **E**phemeral reference (i.e.
/// with a lifetime that is not `'static`), a **S**tatic reference
/// and an **O**wned value.
///
/// All three of the type parameters can be [`An<T>`] or [`No<T>`],
/// which allows to construct subsets of the full functionality,
/// and to statically keep track of which variants may exist at
/// any given point in the code.
///
/// If the parameter types statically specify that
/// only one variant can have a value, e.g. `Eso<An<E>, No<S>, No<O>>`
/// then the runtime representation should be as efficient as that of the
/// sole present type.
///
/// Many type signatures in the `impl`s use the [type aliases defined in the
/// `x` module](x) to express how the type of the resulting `Eso` is
/// related to the type parameters of the input `Eso`.
#[derive(Debug, Clone)]
pub enum Eso<E, S, O> {
    /// An ephemeral value
    E(E),
    /// A shared or static value, meaning that client code can hold on
    /// to an `S` without being limited by any given lifetime.
    S(S),
    /// An owned value, meaning that client code has sole possession of
    /// the contained object (albeit it may be borrowed out by reference)
    O(O),
}

/// An [`Eso`] of [`Eso`]s, but the inner [`Eso`]s are constrained
/// to definitely contain the corresponding varient.
pub type ConstrainedEsoOfEso<E, S, O> = Eso<x::E<E, S, O>, x::S<E, S, O>, x::O<E, S, O>>;

/// Functions to create new [`Eso`]s
mod create;

/// Functions to access the referenced object
mod inside;

/// Functions to manipulate the contained refrerences/values
mod manipulate;

/// Functions to analyze the [`Eso`] and prove those results on a type level
mod prove;

/// Functions to ask about the state of an [`Eso`]
mod query;

/// Functions to change the state of an [`Eso`]
mod transform;

pub mod req;
