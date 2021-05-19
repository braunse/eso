// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Type-level machinery to allow [`Eso::unify`] and [`Eso::select`]
//! to work.
//!
//! The [`Unify`] trait specifies the rules how two [`Maybe`]
//! types can be merged:
//!
//! | This ...    | ... unifies with ... | ... containing ... | ... producing ... |
//! |-------------|----------------------|--------------------|-------------------|
//! | [`An<A>`]   | [`An<A>`]            | same inner type    | [`An<A>`]         |
//! | [`No<A>`]   | [`An<B>`]            | any type           | [`An<B>`]         |
//! | [`An<A>`]   | [`No<B>`]            | any type           | [`An<A>`]         |
//! | [`No<A>`]   | [`No<B>`]            | any type           | [`No<A>`]         |
//!
//! This module also provides the utility trait [`Unify3`] that
//! contains the ugly type manipulations to apply the [`Unify`] rules
//! between three types.

use crate::eso::Eso;
use crate::maybe::{An, Impossible, No};

/// A type `A` that can be unified with another type `B`.
///
/// ## `Maybe`s
///
/// Possible values can be unified with possible values of the
/// same type:
///
/// ```
/// # use eso::maybe::*;
/// # use eso::unify::*;
/// let one: An<i32> = <An<i32> as Unify<An<i32>>>::inject_a(An(1));
/// let two: An<i32> = <An<i32> as Unify<An<i32>>>::inject_b(An(1));
/// ```
///
/// When selecting between a possible and an impossible value,
/// the result must be the possible value:
///
/// ```
/// # use eso::maybe::*;
/// # use eso::unify::*;
/// type Yes = An<i32>;
/// type Nope = No<i32>;
/// type Merged = <Yes as Unify<Nope>>::Out;
/// let yes: Yes = An(1);
/// let merged: An<i32> = <Yes as Unify<Nope>>::inject_a(yes);
///
/// // reflectively:
/// let yes: Yes = An(1);
/// let merged: An<i32> = <Nope as Unify<Yes>>::inject_b(yes);
/// ```
///
/// The inner type of a [`No`] does not matter, it will always
/// unify to the [`An`]:
///
/// ```
/// # use eso::maybe::*;
/// # use eso::unify::*;
/// type Yes = An<i32>;
/// type Nope = No<String>;
/// type Merged = <Yes as Unify<Nope>>::Out;
/// let yes: Yes = An(1);
/// let merged: An<i32> = <Yes as Unify<Nope>>::inject_a(yes);
///
/// // reflectively:
/// let yes: Yes = An(1);
/// let merged: An<i32> = <Nope as Unify<Yes>>::inject_b(yes);
/// ```
///
/// Two [`No`]s will arbitrarily unify to the one on which this
/// trait is invoked. The asymmetry does not matter however,
/// because a [`No`] can always be converted to any other type
/// via [`Impossible::absurd`]:
///
/// ```
/// # use eso::maybe::*;
/// # use eso::unify::*;
/// fn merge_nopes_a(a: No<i32>, b: No<String>) -> No<i32> {
///     <No<i32> as Unify<No<String>>>::inject_a(a)
/// }
///
/// fn merge_nopes_b(a: No<i32>, b: No<String>) -> No<i32> {
///     <No<i32> as Unify<No<String>>>::inject_b(b)
/// }
/// ```
///
/// ## `Eso`s
///
/// Two [`Eso`]s can be unified, if their respective `E`, `S` and `O`
/// type parameters can be unified:
///
/// ```
/// # use eso::{eso::*, maybe::*, unify::*, shorthand::t};
/// type ESO1 = t::S<&'static str, &'static str, String>;
/// type ESO2 = t::O<&'static str, &'static str, String>;
/// type Merged = t::SO<&'static str, &'static str, String>;
/// let eso1: Merged = <ESO1 as Unify<ESO2>>::inject_a(ESO1::from_static("Hello"));
/// let eso2: Merged = <ESO1 as Unify<ESO2>>::inject_b(ESO2::from_owned("Hello".to_string()));
/// ```
///
pub trait Unify<B> {
    /// The result of unifying `Self` and `B`
    type Out;

    /// Make an `Out` value, given a value of type `Self`
    fn inject_a(self) -> Self::Out;

    /// Make an `Out` value, given a value of type `B`
    fn inject_b(b: B) -> Self::Out;
}

impl<T, U> Unify<An<T>> for No<U> {
    type Out = An<T>;

    fn inject_a(self) -> Self::Out {
        self.absurd()
    }

    fn inject_b(b: An<T>) -> Self::Out {
        b
    }
}

impl<T, U> Unify<No<U>> for An<T> {
    type Out = An<T>;

    fn inject_a(self) -> Self::Out {
        self
    }

    fn inject_b(b: No<U>) -> Self::Out {
        b.absurd()
    }
}

impl<T> Unify<An<T>> for An<T> {
    type Out = An<T>;

    fn inject_a(self) -> Self::Out {
        self
    }

    fn inject_b(b: An<T>) -> Self::Out {
        b
    }
}

impl<T, U> Unify<No<U>> for No<T> {
    type Out = No<T>;

    fn inject_a(self) -> Self::Out {
        self
    }

    fn inject_b(b: No<U>) -> Self::Out {
        b.absurd()
    }
}

impl<AE, AS, AO, BE, BS, BO> Unify<Eso<BE, BS, BO>> for Eso<AE, AS, AO>
where
    AE: Unify<BE>,
    AS: Unify<BS>,
    AO: Unify<BO>,
{
    type Out = Eso<AE::Out, AS::Out, AO::Out>;

    fn inject_a(self) -> Self::Out {
        self.outer_map(AE::inject_a, AS::inject_a, AO::inject_a)
    }

    fn inject_b(b: Eso<BE, BS, BO>) -> Self::Out {
        b.outer_map(AE::inject_b, AS::inject_b, AO::inject_b)
    }
}

/// Shorthand for unifying three types by applying [`Unify`]
/// twice.
pub trait Unify3<B, C> {
    /// The resulting type when unifying `Self`, `B`, and `C`
    type Out3;

    /// Make an `Out3` value, given a value of type `Self`
    fn inject3_a(self) -> Self::Out3;

    /// Make an `Out3` value, given a value of type `B`
    fn inject3_b(b: B) -> Self::Out3;

    /// Make an `Out3` value, given a value of type `C`
    fn inject3_c(c: C) -> Self::Out3;
}

impl<A, B, C> Unify3<B, C> for A
where
    A: Unify<B>,
    A::Out: Unify<C>,
{
    type Out3 = <A::Out as Unify<C>>::Out;

    fn inject3_a(self) -> Self::Out3 {
        let ab = A::inject_a(self);
        A::Out::inject_a(ab)
    }

    fn inject3_b(b: B) -> Self::Out3 {
        let ab = A::inject_b(b);
        A::Out::inject_a(ab)
    }

    fn inject3_c(c: C) -> Self::Out3 {
        A::Out::inject_b(c)
    }
}
