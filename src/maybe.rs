// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The [`Maybe`] trait and its two implementations [`An`] and [`No`]
//! describe a compile-time optional value.

use std::{
    marker::PhantomData,
    panic::{RefUnwindSafe, UnwindSafe},
};

/// Prevent anyone else from implementing our traits.
/// This module is unexported so no other crate can
/// implement the [`__SealMaybe`] trait.
mod sealed {
    /// A marker trait that marks types as originating from
    /// this crate. Cannot be implemented outside of our crate.
    pub trait __SealMaybe {}
}

use sealed::*;

/// A type-level optional value.
///
/// This trait is sealed and is only implemented by two types from
/// this crate:
///
/// - [`An<T>`] denotes a value that is present on the type-level
/// - [`No<T>`] denotes the absence of any value, as in there can
///   never be an expression that is executed at run-time that
///   yields a value of this type.
///
/// Note that since no value of [`No`] can ever exist, all functions
/// in this trait may safely assume that they will only be called
/// on an [`An`].
///
/// ```
/// # use eso::maybe::{An,Impossible,No};
/// pub enum X {
///     Yup(An<i32>),
///     Nope(No<String>),
/// }
///
/// // Since no value of X can be of the `Nope` variant, this is
/// // well-typed and cannot panic:
/// fn unwrap_x(x: X) -> i32 {
///     match x {
///         X::Yup(An(i)) => i,
///         _ => unreachable!()
///     }
/// }
/// ```
pub trait Maybe: Sized + __SealMaybe {
    /// The type whose presence or absence is in question
    type Inner;

    /// Yield a reference to the inner value.
    fn inner(&self) -> &Self::Inner;

    /// Yield a mutable reference to the inner value.
    fn inner_mut(&mut self) -> &mut Self::Inner;

    /// Recover the inner value from the wrapper.
    fn unwrap(self) -> Self::Inner;

    /// Run a function on the inner value that either succeeds
    /// or gives back the inner value
    fn unwrap_try<F, T>(self, f: F) -> Result<T, Self>
    where
        F: FnOnce(Self::Inner) -> Result<T, Self::Inner>;

    /// Run a function on the inner value that either succeeds
    /// or gives back the inner value, plus an error value
    fn unwrap_try_x<F, T, E>(self, f: F) -> Result<T, (Self, E)>
    where
        F: FnOnce(Self::Inner) -> Result<T, (Self::Inner, E)>;

    /// Apply a function to the contained value while keeping
    /// the result contained.
    fn map<F, NewInner>(self, f: F) -> <Self as MaybeMap<NewInner>>::Out
    where
        Self: MaybeMap<NewInner>,
        F: FnOnce(Self::Inner) -> NewInner,
    {
        <Self as MaybeMap<NewInner>>::do_map(self, f)
    }
}

/// A type-level function to describe the result
/// of a [`Maybe::map`] operation
pub trait MaybeMap<NewInner>: Maybe {
    /// A [`Maybe`] with the inner type replaced by `NewInner`
    type Out: Maybe<Inner = NewInner>;

    /// The `self` is required as evidence that
    /// you are not constructing a [`No`].
    fn do_map<F>(self, f: F) -> Self::Out
    where
        F: FnOnce(Self::Inner) -> NewInner;
}

/// A trait characterizing a never-existing value
pub trait Impossible {
    /// Conjure up anything from the nonexistant value.
    ///
    /// Since `self` in this function can never exist, it follows
    /// that this function can never be called.
    /// Which means that we can pretend that it returns whatever
    /// the situation calls for.
    /// The code path in which it is called can not be executed
    /// anyway, but still has to typecheck.
    fn absurd<T>(&self) -> T;
}

/// A value of type `A` that exists.
///
/// See the notes about [`Maybe`] for a deeper explanation.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct An<A>(pub A);

impl<A> __SealMaybe for An<A> {}

impl<A> Maybe for An<A> {
    type Inner = A;

    fn inner(&self) -> &A {
        &self.0
    }

    fn inner_mut(&mut self) -> &mut A {
        &mut self.0
    }

    fn unwrap(self) -> A {
        self.0
    }

    fn unwrap_try<F, T>(self, f: F) -> Result<T, Self>
    where
        F: FnOnce(Self::Inner) -> Result<T, A>,
    {
        match f(self.unwrap()) {
            Ok(r) => Ok(r),
            Err(inner) => Err(An(inner)),
        }
    }

    fn unwrap_try_x<F, T, E>(self, f: F) -> Result<T, (Self, E)>
    where
        F: FnOnce(Self::Inner) -> Result<T, (Self::Inner, E)>,
    {
        match f(self.unwrap()) {
            Ok(r) => Ok(r),
            Err((inner, e)) => Err((An(inner), e)),
        }
    }

    fn map<F, B>(self, f: F) -> <Self as MaybeMap<B>>::Out
    where
        Self: MaybeMap<B>,
        F: FnOnce(Self::Inner) -> B,
    {
        <Self as MaybeMap<B>>::do_map(self, f)
    }
}

impl<A, B> MaybeMap<B> for An<A> {
    type Out = An<B>;

    #[inline]
    fn do_map<F>(self, f: F) -> Self::Out
    where
        F: FnOnce(Self::Inner) -> B,
    {
        An(f(self.0))
    }
}

#[derive(Debug, Clone)]
enum Nothing {}

/// A value of type `A` that cannot exist.
///
/// See the notes about [`Maybe`] for a deeper explanation.
#[derive(Debug)]
pub struct No<A> {
    ghost: PhantomData<A>,
    impossible: Nothing,
}

impl<A> Impossible for No<A> {
    fn absurd<T>(&self) -> T {
        match self.impossible {}
    }
}

impl<A> __SealMaybe for No<A> {}

impl<A> Maybe for No<A> {
    type Inner = A;

    fn inner(&self) -> &A {
        self.absurd()
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        self.absurd()
    }

    fn unwrap(self) -> A {
        self.absurd()
    }

    fn unwrap_try<F, T>(self, _f: F) -> Result<T, Self>
    where
        F: FnOnce(Self::Inner) -> Result<T, A>,
    {
        self.absurd()
    }

    fn unwrap_try_x<F, T, E>(self, _f: F) -> Result<T, (Self, E)>
    where
        F: FnOnce(Self::Inner) -> Result<T, (Self::Inner, E)>,
    {
        self.absurd()
    }

    fn map<F, NewInner>(self, _: F) -> <Self as MaybeMap<NewInner>>::Out
    where
        Self: MaybeMap<NewInner>,
        F: FnOnce(Self::Inner) -> NewInner,
    {
        self.absurd()
    }
}

impl<A, B> MaybeMap<B> for No<A> {
    type Out = No<B>;

    fn do_map<F>(self, _: F) -> Self::Out
    where
        F: FnOnce(Self::Inner) -> B,
    {
        self.absurd()
    }
}

impl<A> Clone for No<A> {
    fn clone(&self) -> Self {
        self.absurd()
    }
}

/// **SAFETY**: Since you can't get hold of a [`No<A>`] anyway, and therefore can't ever
/// have a reference to one (in safe code), any code path where you hold one
/// across a potential panic is dead and won't ever execute.
impl<A> RefUnwindSafe for No<A> {}

/// **SAFETY**: Since you can't get hold of a [`No<A>`] anyway, it doesn't matter at all
/// whether or not you try to send one to another thread.
#[cfg(feature = "allow-unsafe")]
#[allow(unsafe_code)]
unsafe impl<A> Send for No<A> {}

/// **SAFETY**: Since you can't get hold of a [`No<A>`] anyway, and therefore can't ever
/// have a reference to one (in safe code), it doesn't matter at all, *how many
/// threads* you can't ever have those references on.
#[cfg(feature = "allow-unsafe")]
#[allow(unsafe_code)]
unsafe impl<A> Sync for No<A> {}

/// **SAFETY**: Since you can't get hold of a [`No<A>`] anyway, whatever detrimental
/// effects might arise from unpinning it can never happen since it's not there
/// in the first place, and the code unpinning it will never execute.
impl<A> Unpin for No<A> {}

/// **SAFETY**: Since you can't get hold of a [`No<A>`] anyway, any code path where you
/// hold one across a potential panic is dead and will never execute.
impl<A> UnwindSafe for No<A> {}

/// Safe conversion between [`Maybe`]s.
///
/// Casting between [`An`] and [`No`] is safe in the following
/// combinations:
///
///  - An [`An<A>`] may only be cast into itself.
///
///  - A [`No<A>`] may be cast into any [`An<B>`] or any other
///    [`No<B>`], since the [`No<A>`] cannot exist in the
///    first place, so the cast can never actually happen.
pub trait Relax<Into>: Maybe {
    /// Cast `self` into another type of [`Maybe`].
    ///
    /// See the [trait documentation](Relax) for the rules.
    fn relax(self) -> Into;
}

impl<A> Relax<An<A>> for An<A> {
    fn relax(self) -> An<A> {
        self
    }
}

impl<A, B> Relax<An<B>> for No<A> {
    fn relax(self) -> An<B> {
        self.absurd()
    }
}

impl<A, B> Relax<No<B>> for No<A> {
    fn relax(self) -> No<B> {
        self.absurd()
    }
}
