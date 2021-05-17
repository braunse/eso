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

// You have been warned:
// #![allow(clippy::clippy::type_complexity)]

use crate::{
    borrow::Reborrowable,
    maybe::{An, Impossible, Maybe, MaybeMap, No, Relax},
    unify::Unify3,
};
use std::borrow::Cow;

use req::{MBorrowable, MOwnableRef, MReborrowable, MUnwrapInto};

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

impl<E, MS, MO> Eso<An<E>, MS, MO> {
    /// Create an [`Eso`] from a reference
    pub const fn from_ref(e: E) -> Self {
        Eso::E(An(e))
    }
}

impl<ME, S, MO> Eso<ME, An<S>, MO> {
    /// Create an [`Eso`] from a shared/static reference
    pub const fn from_static(s: S) -> Self {
        Eso::S(An(s))
    }
}

impl<ME, MS, O> Eso<ME, MS, An<O>> {
    /// Create an [`Eso`] from an owned object
    pub const fn from_owned(o: O) -> Self {
        Eso::O(An(o))
    }

    /// Get a mutable reference to the contained owned value,
    /// cloning out of a referenced object if necessary.
    pub fn to_mut<'a>(&'a mut self) -> &'a mut O
    where
        ME: MOwnableRef<'a, O> + Clone,
        MS: MOwnableRef<'a, O> + Clone,
    {
        match self {
            Eso::E(e) => {
                *self = Eso::O(An(e.to_owned()));
                match self {
                    Eso::O(An(o)) => o,
                    _ => unreachable!(), // we just assigned it
                }
            }
            Eso::S(s) => {
                *self = Eso::O(An(s.to_owned()));
                match self {
                    Eso::O(An(o)) => o,
                    _ => unreachable!(), // we just assigned it
                }
            }
            Eso::O(An(o)) => o,
        }
    }
}

impl<E, MS, O> Eso<An<E>, MS, An<O>> {
    /// Create an [`Eso`] from a [`Cow`].
    ///
    /// This will either convert the reference from a [`Cow::Borrowed`]
    /// into the correct generalized reference type via [`Reborrowable`]
    /// or take ownership of the owned value from a [`Cow::Owned`].
    pub fn from_cow<'a, T: ToOwned + ?Sized>(cow: Cow<'a, T>) -> Self
    where
        E: 'a,
        &'a T: Reborrowable<'a, E>,
        O: From<T::Owned>,
    {
        match cow {
            Cow::Borrowed(r) => Eso::E(An(r.reborrow())),
            Cow::Owned(o) => Eso::O(An(o.into())),
        }
    }
}

impl<ME, MS, MO> Eso<ME, MS, MO> {
    /// Returns `true` if the [`Eso`] is of the [`Eso::E`] variant.
    pub fn is_ephemeral(&self) -> bool {
        match self {
            Eso::E(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the [`Eso`] is of the [`Eso::S`] variant.
    pub fn is_static(&self) -> bool {
        match self {
            Eso::E(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the [`Eso`] is of the [`Eso::O`] variant.
    pub fn is_owning(&self) -> bool {
        match self {
            Eso::E(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the [`Eso`] does not own the contained value,
    /// i. e. it is of the [`Eso::E`] or [`Eso::S`] variants.
    pub fn is_reference(&self) -> bool {
        match self {
            Eso::O(_) => false,
            _ => true,
        }
    }

    /// Returns `true` if the [`Eso`] is lasting, i. e. it is not
    /// an ephemeral reference.
    pub fn is_lasting(&self) -> bool {
        match self {
            Eso::E(_) => false,
            _ => true,
        }
    }

    /// Transform this [`Eso`] into one that can only be a static/shared
    /// reference or an owned value.
    ///
    /// This clones an ephemeral reference into an owned value via
    /// [`Ownable`](crate::borrow::Ownable)
    /// but will move a shared/static reference or an owned value into the
    /// result unchanged.
    ///
    /// If there is a non-static lifetime mentioned in the static type of
    /// `E`, this can then be dropped via [`Eso::relax`] to reflect the
    /// *'lifetime-less-ness'* of the result.
    pub fn into_static<'s>(self) -> x::sO<ME, MS, MO>
    where
        ME: MOwnableRef<'s, MO::Inner>,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => Eso::O(An(e.own())),
            Eso::S(s) => Eso::S(s),
            Eso::O(o) => Eso::O(An(o.unwrap())),
        }
    }

    /// Clone this [`Eso`] into one that can only be a static/shared
    /// reference or an owned value.
    ///
    /// This clones an ephemeral reference into an owned value via
    /// [`Ownable`](crate::borrow::Ownable)
    /// but clones a shared/static reference or an owned value into the
    /// result unchanged.
    ///
    /// If there is a non-static lifetime mentioned in the static type of
    /// `E`, this can then be dropped via [`Eso::relax`] to reflect the
    /// *'lifetime-less-ness'* of the result.
    pub fn to_static<'a>(&'a self) -> x::sO<ME, MS, MO>
    where
        ME: MOwnableRef<'a, MO::Inner> + Clone,
        MS: Maybe + Clone,
        MO: Maybe + Clone,
    {
        match self {
            Eso::E(e) => Eso::O(An(e.to_owned())),
            Eso::S(s) => Eso::S(s.clone()),
            Eso::O(o) => Eso::O(An(o.clone().unwrap())),
        }
    }

    /// Transform this [`Eso`] into one that is definitely an owned value.
    ///
    /// Any reference will be cloned into an owned form via
    /// [`Ownable`](crate::borrow::Ownable),
    /// and an owned value will be moved into the result unchanged.
    pub fn into_owning<'s>(self) -> x::O<ME, MS, MO>
    where
        ME: MOwnableRef<'s, MO::Inner>,
        MS: MOwnableRef<'s, MO::Inner>,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => Eso::O(An(e.own())),
            Eso::S(s) => Eso::O(An(s.own())),
            Eso::O(o) => Eso::O(An(o.unwrap())),
        }
    }

    /// Clone this [`Eso`] into one that is definitely an owned value.
    ///
    /// Any reference will be cloned into an owned form via
    /// [`Ownable`](crate::borrow::Ownable),
    /// and an owned value will be cloned into the result via [`Clone`].
    pub fn to_owning<'a>(&'a self) -> x::O<ME, MS, MO>
    where
        ME: MOwnableRef<'a, MO::Inner> + Clone,
        MS: MOwnableRef<'a, MO::Inner> + Clone,
        MO: Maybe + Clone,
    {
        match self {
            Eso::E(e) => Eso::O(An(e.to_owned())),
            Eso::S(s) => Eso::O(An(s.to_owned())),
            Eso::O(o) => Eso::O(An(o.clone().unwrap())),
        }
    }

    /// Borrow a generalized reference of type `T` from this [`Eso`].
    pub fn get_ref<'a, T: 'a>(&'a self) -> T
    where
        ME: Clone + MReborrowable<'a, T>,
        MS: Clone + MReborrowable<'a, T>,
        MO: MBorrowable<'a, T>,
    {
        match self {
            Eso::E(e) => e.clone().reborrow(),
            Eso::S(s) => s.clone().reborrow(),
            Eso::O(o) => o.borrow(),
        }
    }

    /// Mutably borrow the owned value contained in this [`Eso`],
    /// if it actually contains an owned value.
    pub fn try_get_mut(&mut self) -> Option<&mut MO::Inner>
    where
        MO: Maybe,
    {
        match self {
            Eso::E(_) => None,
            Eso::S(_) => None,
            Eso::O(o) => Some(o.inner_mut()),
        }
    }

    /// Match on the ephemeral case, and modify the type parameters to prove
    /// the match, if successful.
    /// Otherwise, casts the type parameters to prove the non-match.
    pub fn try_split_ephemeral(self) -> Result<x::E<ME, MS, MO>, x::so<ME, MS, MO>>
    where
        ME: Maybe,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => Ok(Eso::E(An(e.unwrap()))),
            Eso::S(s) => Err(Eso::S(s)),
            Eso::O(o) => Err(Eso::O(o)),
        }
    }

    /// Match on the static/shared case, and modify the type parameters to prove
    /// the match, if successful.
    /// Otherwise, casts the type parameters to prove the non-match.
    pub fn try_split_static(self) -> Result<x::S<ME, MS, MO>, x::eo<ME, MS, MO>>
    where
        ME: Maybe,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::S(s) => Ok(Eso::S(An(s.unwrap()))),
            Eso::E(e) => Err(Eso::E(e)),
            Eso::O(o) => Err(Eso::O(o)),
        }
    }

    /// Match on the owned case, and modify the type parameters to prove
    /// the match, if successful.
    /// Otherwise, casts the type parameters to prove the non-match.
    pub fn try_split_owned(self) -> Result<x::O<ME, MS, MO>, x::es<ME, MS, MO>>
    where
        ME: Maybe,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::O(o) => Ok(Eso::O(An(o.unwrap()))),
            Eso::E(e) => Err(Eso::E(e)),
            Eso::S(s) => Err(Eso::S(s)),
        }
    }

    /// Retrieve the contained ephemeral value.
    /// If the value is not ephemeral, cast the type parameters to prove the non-match.
    pub fn try_unwrap_ephemeral(self) -> Result<ME::Inner, x::so<ME, MS, MO>>
    where
        ME: Maybe,
    {
        match self {
            Eso::E(e) => Ok(e.unwrap()),
            Eso::S(s) => Err(Eso::S(s)),
            Eso::O(o) => Err(Eso::O(o)),
        }
    }

    /// Retrieve the contained owned value.
    /// If the value is not owned, cast the type parameters to prove the non-match.
    pub fn try_unwrap_owned(self) -> Result<MO::Inner, x::es<ME, MS, MO>>
    where
        MO: Maybe,
    {
        match self {
            Eso::E(e) => Err(Eso::E(e)),
            Eso::S(s) => Err(Eso::S(s)),
            Eso::O(o) => Ok(o.unwrap()),
        }
    }

    /// Retrieve the contained static/shared value.
    /// If the value is not static/shared, cast the type parameters to prove the non-match.
    pub fn try_unwrap_static(self) -> Result<MS::Inner, x::eo<ME, MS, MO>>
    where
        MS: Maybe,
    {
        match self {
            Eso::E(e) => Err(Eso::E(e)),
            Eso::S(s) => Ok(s.unwrap()),
            Eso::O(o) => Err(Eso::O(o)),
        }
    }

    /// Borrow an ephemeral reference or preserve a static/shared reference.
    /// If the [`Eso`] contains an owned value, borrow a reference to it.
    pub fn reference<'a>(&'a self) -> x::ES<ME, MS, MO>
    where
        ME: Maybe + Clone,
        MS: Maybe + Clone,
        MO: MBorrowable<'a, ME::Inner>,
    {
        match self {
            Eso::E(e) => Eso::E(An(e.clone().unwrap())),
            Eso::S(s) => Eso::S(An(s.clone().unwrap())),
            Eso::O(o) => Eso::E(An(o.borrow())),
        }
    }

    /// Borrow an ephemeral reference.
    ///
    /// Clones an already-existing ephemeral reference,
    /// [reborrows](Reborrowable) a shared/static reference or
    /// [borrows](crate::borrow::Borrowable) a generalized reference to
    /// an owned value.
    pub fn ephemeral<'a>(&'a self) -> x::E<ME, MS, MO>
    where
        ME: Maybe + Clone,
        MS: MReborrowable<'a, ME::Inner> + Clone,
        MO: MBorrowable<'a, ME::Inner>,
    {
        match self {
            Eso::E(e) => Eso::E(An(e.clone().unwrap())),
            Eso::S(s) => Eso::E(An(s.clone().reborrow())),
            Eso::O(o) => Eso::E(An(o.borrow())),
        }
    }

    /// Transform into a [`Cow`].
    ///
    /// [Reborrows](Reborrowable) an ephemeral or static/shared reference,
    /// preserves an owned value.
    pub fn into_cow<'a, T: ?Sized + ToOwned + 'a>(self) -> Cow<'a, T>
    where
        ME: MReborrowable<'a, &'a T>,
        MS: MReborrowable<'a, &'a T>,
        MO: MUnwrapInto<T::Owned>,
    {
        match self {
            Eso::E(e) => Cow::Borrowed(e.reborrow()),
            Eso::S(s) => Cow::Borrowed(s.reborrow()),
            Eso::O(o) => Cow::Owned(o.unwrap_into()),
        }
    }

    /// Relax the type parameters to fit an expected combination
    /// of type parameters.
    ///
    /// See [`Relax`] for the rules about relaxation.
    pub fn relax<ME1, MS1, MO1>(self) -> Eso<ME1, MS1, MO1>
    where
        ME: Relax<ME1>,
        MS: Relax<MS1>,
        MO: Relax<MO1>,
    {
        match self {
            Eso::E(e) => Eso::E(e.relax()),
            Eso::S(s) => Eso::S(s.relax()),
            Eso::O(o) => Eso::O(o.relax()),
        }
    }

    /// Clone the [`Eso`] with relaxed type parameters, to fit
    /// an expected configuration.
    ///
    /// See [`Relax`] for the rules about relaxation.
    pub fn clone_relax<ME1, MS1, MO1>(&self) -> Eso<ME1, MS1, MO1>
    where
        ME: Clone + Relax<ME1>,
        MS: Clone + Relax<MS1>,
        MO: Clone + Relax<MO1>,
    {
        match self {
            Eso::E(e) => Eso::E(e.clone().relax()),
            Eso::S(s) => Eso::S(s.clone().relax()),
            Eso::O(o) => Eso::O(o.clone().relax()),
        }
    }

    /// Distinguish the three cases of [`Eso`] while maintaining
    /// the inner [`Eso`]-ness of the resulting values.
    ///
    /// Since [`Eso`] does not implement [`Maybe`], the
    /// return value is pretty much only useful for pattern
    /// matching, as none of the [`Eso`] machinery will be
    /// applicable to the result type.
    pub fn split(self) -> ConstrainedEsoOfEso<ME, MS, MO>
    where
        ME: Maybe,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => Eso::E(Eso::E(An(e.unwrap()))),
            Eso::S(s) => Eso::S(Eso::S(An(s.unwrap()))),
            Eso::O(o) => Eso::O(Eso::O(An(o.unwrap()))),
        }
    }

    // pub fn unsplit<CE, CS, CO>(self) -> Eso<CE, CS, CO>
    // where ME: Maybe<Inner:

    /// Transform `self´ by applying a function to the `E`
    /// variant while preserving the other variants.
    pub fn map_e<F, T>(self, f: F) -> Eso<ME::Out, MS, MO>
    where
        ME: MaybeMap<T>,
        F: FnOnce(ME::Inner) -> T,
    {
        match self {
            Eso::E(e) => Eso::E(e.map(f)),
            Eso::S(s) => Eso::S(s),
            Eso::O(o) => Eso::O(o),
        }
    }

    /// Transform `self´ by applying a function to the `E`
    /// variant while preserving the other variants.
    pub fn map_s<F, T>(self, f: F) -> Eso<ME, MS::Out, MO>
    where
        MS: MaybeMap<T>,
        F: FnOnce(MS::Inner) -> T,
    {
        match self {
            Eso::E(e) => Eso::E(e),
            Eso::S(s) => Eso::S(s.map(f)),
            Eso::O(o) => Eso::O(o),
        }
    }

    /// Transform `self´ by applying a function to the `E`
    /// variant while preserving the other variants.
    pub fn map_o<F, T>(self, f: F) -> Eso<ME, MS, MO::Out>
    where
        MO: MaybeMap<T>,
        F: FnOnce(MO::Inner) -> T,
    {
        match self {
            Eso::E(e) => Eso::E(e),
            Eso::S(s) => Eso::S(s),
            Eso::O(o) => Eso::O(o.map(f)),
        }
    }

    /// Transform `self´ by applying a different function to
    /// each of the variants.
    pub fn map<EF, ET, SF, ST, OF, OT>(
        self,
        ef: EF,
        sf: SF,
        of: OF,
    ) -> Eso<ME::Out, MS::Out, MO::Out>
    where
        ME: MaybeMap<ET>,
        MS: MaybeMap<ST>,
        MO: MaybeMap<OT>,
        EF: FnOnce(ME::Inner) -> ET,
        SF: FnOnce(MS::Inner) -> ST,
        OF: FnOnce(MO::Inner) -> OT,
    {
        match self {
            Eso::E(e) => Eso::E(e.map(ef)),
            Eso::S(s) => Eso::S(s.map(sf)),
            Eso::O(o) => Eso::O(o.map(of)),
        }
    }

    /// Special case of [`merge_with`](Self::merge_with) to match the expected
    /// name for operations that map a contained value into the container type.
    pub fn flat_map<EF, SF, OF, ME1, MS1, MO1>(self, ef: EF, sf: SF, of: OF) -> Eso<ME1, MS1, MO1>
    where
        ME: Maybe,
        MS: Maybe,
        MO: Maybe,
        EF: FnOnce(ME::Inner) -> Eso<ME1, MS1, MO1>,
        SF: FnOnce(MS::Inner) -> Eso<ME1, MS1, MO1>,
        OF: FnOnce(MO::Inner) -> Eso<ME1, MS1, MO1>,
    {
        self.merge_with(ef, sf, of)
    }

    /// Extract the contained value, if all variants may contain the same
    /// type.
    pub fn merge(self) -> ME::Inner
    where
        ME: Maybe,
        MS: Maybe<Inner = ME::Inner>,
        MO: Maybe<Inner = ME::Inner>,
    {
        match self {
            Eso::E(e) => e.unwrap(),
            Eso::S(s) => s.unwrap(),
            Eso::O(o) => o.unwrap(),
        }
    }

    /// Select one of the given functions to run based on which of the
    /// possible values is selected. The function will run on the
    /// inner values of the [`Maybe`]s.
    pub fn merge_with<EF, SF, OF, T>(self, ef: EF, sf: SF, of: OF) -> T
    where
        ME: Maybe,
        MS: Maybe,
        MO: Maybe,
        EF: FnOnce(ME::Inner) -> T,
        SF: FnOnce(MS::Inner) -> T,
        OF: FnOnce(MO::Inner) -> T,
    {
        match self {
            Eso::E(e) => ef(e.unwrap()),
            Eso::S(s) => sf(s.unwrap()),
            Eso::O(o) => of(o.unwrap()),
        }
    }

    /// Select one of the given functions to run based on which of the
    /// possible values is selected. The function will run on the
    /// [`Maybe`]s themselves.
    pub fn outer_map<EF, SF, OF, ME1, MS1, MO1>(self, ef: EF, sf: SF, of: OF) -> Eso<ME1, MS1, MO1>
    where
        EF: FnOnce(ME) -> ME1,
        SF: FnOnce(MS) -> MS1,
        OF: FnOnce(MO) -> MO1,
    {
        match self {
            Eso::E(e) => Eso::E(ef(e)),
            Eso::S(s) => Eso::S(sf(s)),
            Eso::O(o) => Eso::O(of(o)),
        }
    }

    /// Merge the three possible values into one by selecting
    /// from the variants according to the rules of [`unify`](crate::unify)
    ///
    /// ```
    /// # use eso::eso::*;
    /// type E<'a> = t::E<&'a i32, &'static i32, i32>;
    /// type S<'a> = t::S<&'a i32, &'static i32, i32>;
    /// type O<'a> = t::O<&'a i32, &'static i32, i32>;
    /// type Nested<'a> = Eso<E<'a>, S<'a>, O<'a>>;
    /// type Merged<'a> = t::ESO<&'a i32, &'static i32, i32>;
    ///
    /// # fn f<'a>(r: &'a i32) {
    /// let eso1: Merged = Nested::E(E::from_ref(r)).unify();
    /// let eso2: Merged = Nested::S(S::from_static(&12)).unify();
    /// let eso3: Merged = Nested::O(O::from_owned(42)).unify();
    /// assert_eq!(eso1.get_ref::<&i32>(), r);
    /// assert_eq!(eso2.get_ref::<&i32>(), &12);
    /// assert_eq!(eso3.get_ref::<&i32>(), &42);
    /// # }
    /// # f(&1);
    /// ```
    ///
    /// As in [`unify`](crate::unify), any [`No`] values can be
    /// unified with anything else:
    ///
    /// ```
    /// # use eso::eso::*;
    /// # struct T1; struct T2; struct T3; struct T4; struct T5; struct T6;
    /// type E = t::E<i32, T1, T2>;
    /// type S = t::S<T3, i32, T4>;
    /// type O = t::O<T5, T6, i32>;
    /// type Nested = Eso<E, S, O>;
    /// type Merged = t::ESO<i32, i32, i32>;
    ///
    /// let eso1: Merged = Nested::E(E::from_ref(1)).unify();
    /// let eso2: Merged = Nested::S(S::from_static(2)).unify();
    /// let eso3: Merged = Nested::O(O::from_owned(3)).unify();
    /// assert_eq!(eso1.merge(), 1);
    /// assert_eq!(eso2.merge(), 2);
    /// assert_eq!(eso3.merge(), 3);
    /// ```
    pub fn unify(self) -> ME::Out3
    where
        ME: Unify3<MS, MO>,
    {
        match self {
            Eso::E(e) => ME::inject3_a(e),
            Eso::S(s) => ME::inject3_b(s),
            Eso::O(o) => ME::inject3_c(o),
        }
    }
}

impl<E, S, O> Eso<An<E>, No<S>, No<O>> {
    /// Safely move the ephemeral reference out of this [`Eso`].
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an ephemeral reference, and thus cannot fail.
    pub fn safe_unwrap_ephemeral(self) -> E {
        match self {
            Eso::E(An(e)) => e,
            Eso::S(s) => s.absurd(),
            Eso::O(o) => o.absurd(),
        }
    }
}

impl<E, S, O> Eso<No<E>, No<S>, An<O>> {
    /// Safely move the owned value out of this [`Eso`].
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an owned value, and thus cannot fail.
    pub fn safe_unwrap_owned(self) -> O {
        match self {
            Eso::E(e) => e.absurd(),
            Eso::S(s) => s.absurd(),
            Eso::O(An(o)) => o,
        }
    }

    /// Safely reference the owned value contained in this [`Eso`].
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an ephemeral reference, and thus cannot fail.
    pub fn get_owned_ref(&self) -> &O {
        match self {
            Eso::E(e) => e.absurd(),
            Eso::S(s) => s.absurd(),
            Eso::O(An(o)) => o,
        }
    }

    /// Safely and mutably reference the owned value contained in this [`Eso`].
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an ephemeral reference, and thus cannot fail.
    pub fn get_mut(&mut self) -> &mut O {
        match self {
            Eso::E(e) => e.absurd(),
            Eso::S(s) => s.absurd(),
            Eso::O(o) => o.inner_mut(),
        }
    }
}

impl<E, S, O> Eso<No<E>, An<S>, No<O>> {
    /// Safely move the static/shared reference out of this [`Eso`].
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain a static/shared reference, and thus cannot fail.
    pub fn safe_unwrap_static(self) -> S {
        match self {
            Eso::S(An(s)) => s,
            Eso::E(e) => e.absurd(),
            Eso::O(o) => o.absurd(),
        }
    }
}

impl<E, S, O, ES, EO, SE, SO, OE, OS> Eso<t::E<E, ES, EO>, t::S<SE, S, SO>, t::O<OE, OS, O>> {
    /// Undo a [`split`](Eso::split). This is a constrained special case
    /// of [`Eso::unify`].
    pub fn unsplit(self) -> t::ESO<E, S, O> {
        self.unify()
    }
}

impl<E, S, O> Impossible for Eso<No<E>, No<S>, No<O>> {
    fn absurd<T>(&self) -> T {
        match self {
            Eso::E(e) => e.absurd(),
            Eso::S(s) => s.absurd(),
            Eso::O(o) => o.absurd(),
        }
    }
}

impl<'a, T, E, MS, MO> From<&'a T> for Eso<An<E>, MS, MO>
where
    E: 'a,
    &'a T: Reborrowable<'a, E>,
{
    fn from(r: &'a T) -> Self {
        Eso::E(An(r.reborrow()))
    }
}

impl<'a, T: ToOwned, MS> From<Cow<'a, T>> for Eso<An<&'a T>, MS, An<T::Owned>> {
    fn from(it: Cow<'a, T>) -> Self {
        match it {
            Cow::Borrowed(b) => Eso::from_ref(b),
            Cow::Owned(o) => Eso::from_owned(o),
        }
    }
}

/// Shorthand type aliases for [`Eso`].
///
/// The type names derive from the three components `E`, `S`, `O`,
/// but the input type arguments are transformed according to the
/// rules:
///
/// | Rule      | Type parameter  | Transformed to | Meaning      |
/// |-----------|-----------------|----------------|--------------|
/// | Uppercase | `T`: value      | `An<T>`        | present      |
/// | Missing   | `T`: value      | `No<T>`        | absent       |
/// | Lowercase | `MT`: [`Maybe`] | `MT`           | pass-through |
#[allow(
    non_camel_case_types,
    missing_docs,
    clippy::clippy::upper_case_acronyms
)]
pub mod t {
    use crate::maybe::{An, No};

    /// [`Eso`] with `E` present, `S` present, `O` present, see [shorthand module docs](super::t)
    pub type ESO<E, S, O> = super::Eso<An<E>, An<S>, An<O>>;

    /// [`Eso`] with `E` present, `S` present, `O` pass-through, see [shorthand module docs](super::t)
    pub type ESo<E, S, MO> = super::Eso<An<E>, An<S>, MO>;

    /// [`Eso`] with `E` present, `S` pass-through, `O` present, see [shorthand module docs](super::t)
    pub type EsO<E, MS, O> = super::Eso<An<E>, MS, An<O>>;

    /// [`Eso`] with `E` pass-through, `S` present, `O` present, see [shorthand module docs](super::t)
    pub type eSO<ME, S, O> = super::Eso<ME, An<S>, An<O>>;

    /// [`Eso`] with `E` present, `S` pass-through, `O` pass-through, see [shorthand module docs](super::t)
    pub type Eso<E, MS, MO> = super::Eso<An<E>, MS, MO>;

    /// [`Eso`] with `E` pass-through, `S` present, `O` pass-through, see [shorthand module docs](super::t)
    pub type eSo<ME, S, MO> = super::Eso<ME, An<S>, MO>;

    /// [`Eso`] with `E` pass-through, `S` pass-through, `O` present, see [shorthand module docs](super::t)
    pub type esO<ME, MS, O> = super::Eso<ME, MS, An<O>>;

    /// [`Eso`] with `E` pass-through, `S` pass-through, `O` pass-through - actually an alias for [`Eso`], see [shorthand module docs](super::t)
    pub type eso<ME, MS, MO> = super::Eso<ME, MS, MO>;

    /// [`Eso`] with `E` present, `S` present, `O` absent, see [shorthand module docs](super::t)
    pub type ES<E, S, O> = super::Eso<An<E>, An<S>, No<O>>;

    /// [`Eso`] with `E` present, `S` pass-through, `O` absent, see [shorthand module docs](super::t)
    pub type Es<E, MS, O> = super::Eso<An<E>, MS, No<O>>;

    /// [`Eso`] with `E` pass-through, `S` present, `O` absent, see [shorthand module docs](super::t)
    pub type eS<ME, S, O> = super::Eso<ME, An<S>, No<O>>;

    /// [`Eso`] with `E` pass-through, `S` pass-through, `O` absent, see [shorthand module docs](super::t)
    pub type es<ME, MS, O> = super::Eso<ME, MS, No<O>>;

    /// [`Eso`] with `E` present, `S` absent, `O` present, see [shorthand module docs](super::t)
    pub type EO<E, S, O> = super::Eso<An<E>, No<S>, An<O>>;

    /// [`Eso`] with `E` present, `S` absent, `O` pass-through, see [shorthand module docs](super::t)
    pub type Eo<E, S, MO> = super::Eso<An<E>, No<S>, MO>;

    /// [`Eso`] with `E` pass-through, `S` absent, `O` present, see [shorthand module docs](super::t)
    pub type eO<ME, S, O> = super::Eso<ME, No<S>, An<O>>;

    /// [`Eso`] with `E` pass-through, `S` absent, `O` pass-through, see [shorthand module docs](super::t)
    pub type eo<ME, S, MO> = super::Eso<ME, No<S>, MO>;

    /// [`Eso`] with `E` absent, `S` present, `O` present, see [shorthand module docs](super::t)
    pub type SO<E, S, O> = super::Eso<No<E>, An<S>, An<O>>;

    /// [`Eso`] with `E` absent, `S` present, `O` pass-through, see [shorthand module docs](super::t)
    pub type So<E, S, MO> = super::Eso<No<E>, An<S>, MO>;

    /// [`Eso`] with `E` absent, `S` pass-through, `O` present, see [shorthand module docs](super::t)
    pub type sO<E, MS, O> = super::Eso<No<E>, MS, An<O>>;

    /// [`Eso`] with `E` absent, `S` pass-through, `O` pass-through, see [shorthand module docs](super::t)
    pub type so<E, MS, MO> = super::Eso<No<E>, MS, MO>;

    /// [`Eso`] with `E` present, `S` absent, `O` absent, see [shorthand module docs](super::t)
    pub type E<E, S, O> = super::Eso<An<E>, No<S>, No<O>>;

    /// [`Eso`] with `E` pass-through, `S` absent, `O` absent, see [shorthand module docs](super::t)
    pub type e<ME, S, O> = super::Eso<ME, No<S>, No<O>>;

    /// [`Eso`] with `E` absent, `S` present, `O` absent, see [shorthand module docs](super::t)
    pub type S<E, S, O> = super::Eso<No<E>, An<S>, No<O>>;

    /// [`Eso`] with `E` absent, `S` pass-through, `O` absent, see [shorthand module docs](super::t)
    pub type s<E, MS, O> = super::Eso<No<E>, MS, No<O>>;

    /// [`Eso`] with `E` absent, `S` absent, `O` present, see [shorthand module docs](super::t)
    pub type O<E, S, O> = super::Eso<No<E>, No<S>, An<O>>;

    /// [`Eso`] with `E` absent, `S` absent, `O` pass-through, see [shorthand module docs](super::t)
    pub type o<E, S, MO> = super::Eso<No<E>, No<S>, MO>;

    /// [`Eso`] with `E` absent, `S` absent, `O` absent - this is [`Impossible`](crate::maybe::Impossible), see [shorthand module docs](super::t)
    pub type None<E, S, O> = super::Eso<No<E>, No<S>, No<O>>;
}

/// Shorthand type aliases for transformations of an [`Eso`].
///
/// The type names derive from the three components `ME`, `MS`, `MO`,
/// but the input type arguments are transformed according to the
/// rules:
///
/// | Rule      | Type parameter  | Transformed to | Meaning      |
/// |-----------|-----------------|----------------|--------------|
/// | Uppercase | `MT`: [`Maybe`] | `An<T::Inner>` | present      |
/// | Missing   | `MT`: [`Maybe`] | `No<T::Inner>` | absent       |
/// | Lowercase | `MT`: [`Maybe`] | `MT`           | pass-through |
#[allow(
    non_camel_case_types,
    missing_docs,
    clippy::clippy::upper_case_acronyms
)]
pub mod x {
    use crate::maybe::{An, Maybe, No};

    /// [`Eso`] with `E` present, `S` present, `O` present, see [shorthand module docs](super::x)
    pub type ESO<ME, MS, MO> =
        super::Eso<An<<ME as Maybe>::Inner>, An<<MS as Maybe>::Inner>, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` present, `S` present, `O` pass-through, see [shorthand module docs](super::x)
    pub type ESo<ME, MS, MO> = super::Eso<An<<ME as Maybe>::Inner>, An<<MS as Maybe>::Inner>, MO>;

    /// [`Eso`] with `E` present, `S` pass-through, `O` present, see [shorthand module docs](super::x)
    pub type EsO<ME, MS, MO> = super::Eso<An<<ME as Maybe>::Inner>, MS, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` pass-through, `S` present, `O` present, see [shorthand module docs](super::x)
    pub type eSO<ME, MS, MO> = super::Eso<ME, An<<MS as Maybe>::Inner>, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` present, `S` pass-through, `O` pass-through, see [shorthand module docs](super::x)
    pub type Eso<ME, MS, MO> = super::Eso<An<<ME as Maybe>::Inner>, MS, MO>;

    /// [`Eso`] with `E` pass-through, `S` present, `O` pass-through, see [shorthand module docs](super::x)
    pub type eSo<ME, MS, MO> = super::Eso<ME, An<<MS as Maybe>::Inner>, MO>;

    /// [`Eso`] with `E` pass-through, `S` pass-through, `O` present, see [shorthand module docs](super::x)
    pub type esO<ME, MS, MO> = super::Eso<ME, MS, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` pass-through, `S` pass-through, `O` pass-through - actually an alias for [`Eso`], see [shorthand module docs](super::x)
    pub type eso<ME, MS, MO> = super::Eso<ME, MS, MO>;

    /// [`Eso`] with `E` present, `S` present, `O` absent, see [shorthand module docs](super::x)
    pub type ES<ME, MS, MO> =
        super::Eso<An<<ME as Maybe>::Inner>, An<<MS as Maybe>::Inner>, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` present, `S` pass-through, `O` absent, see [shorthand module docs](super::x)
    pub type Es<ME, MS, MO> = super::Eso<An<<ME as Maybe>::Inner>, MS, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` pass-through, `S` present, `O` absent, see [shorthand module docs](super::x)
    pub type eS<ME, MS, MO> = super::Eso<ME, An<<MS as Maybe>::Inner>, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` pass-through, `S` pass-through, `O` absent, see [shorthand module docs](super::x)
    pub type es<ME, MS, MO> = super::Eso<ME, MS, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` present, `S` absent, `O` present, see [shorthand module docs](super::x)
    pub type EO<ME, MS, MO> =
        super::Eso<An<<ME as Maybe>::Inner>, No<<MS as Maybe>::Inner>, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` present, `S` absent, `O` pass-through, see [shorthand module docs](super::x)
    pub type Eo<ME, MS, MO> = super::Eso<An<<ME as Maybe>::Inner>, No<<MS as Maybe>::Inner>, MO>;

    /// [`Eso`] with `E` pass-through, `S` absent, `O` present, see [shorthand module docs](super::x)
    pub type eO<ME, MS, MO> = super::Eso<ME, No<<MS as Maybe>::Inner>, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` pass-through, `S` absent, `O` pass-through, see [shorthand module docs](super::x)
    pub type eo<ME, MS, MO> = super::Eso<ME, No<<MS as Maybe>::Inner>, MO>;

    /// [`Eso`] with `E` absent, `S` present, `O` present, see [shorthand module docs](super::x)
    pub type SO<ME, MS, MO> =
        super::Eso<No<<ME as Maybe>::Inner>, An<<MS as Maybe>::Inner>, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` absent, `S` present, `O` pass-through, see [shorthand module docs](super::x)
    pub type So<ME, MS, MO> = super::Eso<No<<ME as Maybe>::Inner>, An<<MS as Maybe>::Inner>, MO>;

    /// [`Eso`] with `E` absent, `S` pass-through, `O` present, see [shorthand module docs](super::x)
    pub type sO<ME, MS, MO> = super::Eso<No<<ME as Maybe>::Inner>, MS, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` absent, `S` pass-through, `O` pass-through, see [shorthand module docs](super::x)
    pub type so<ME, MS, MO> = super::Eso<No<<ME as Maybe>::Inner>, MS, MO>;

    /// [`Eso`] with `E` present, `S` absent, `O` absent, see [shorthand module docs](super::x)
    pub type E<ME, MS, MO> =
        super::Eso<An<<ME as Maybe>::Inner>, No<<MS as Maybe>::Inner>, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` pass-through, `S` absent, `O` absent, see [shorthand module docs](super::x)
    pub type e<ME, MS, MO> = super::Eso<ME, No<<MS as Maybe>::Inner>, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` absent, `S` present, `O` absent, see [shorthand module docs](super::x)
    pub type S<ME, MS, MO> =
        super::Eso<No<<ME as Maybe>::Inner>, An<<MS as Maybe>::Inner>, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` absent, `S` pass-through, `O` absent, see [shorthand module docs](super::x)
    pub type s<ME, MS, MO> = super::Eso<No<<ME as Maybe>::Inner>, MS, No<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` absent, `S` absent, `O` present, see [shorthand module docs](super::x)
    pub type O<ME, MS, MO> =
        super::Eso<No<<ME as Maybe>::Inner>, No<<MS as Maybe>::Inner>, An<<MO as Maybe>::Inner>>;

    /// [`Eso`] with `E` absent, `S` absent, `O` pass-through, see [shorthand module docs](super::x)
    pub type o<ME, MS, MO> = super::Eso<No<<ME as Maybe>::Inner>, No<<MS as Maybe>::Inner>, MO>;

    /// [`Eso`] with `E` absent, `S` absent, `O` absent - this is [`Impossible`](crate::maybe::Impossible), see [shorthand module docs](super::x)
    pub type None<ME, MS, MO> =
        super::Eso<No<<ME as Maybe>::Inner>, No<<MS as Maybe>::Inner>, No<<MO as Maybe>::Inner>>;
}

/// Shorthand traits for requirements on [`Eso`]s
/// to keep the `where` clauses short and more readable
pub mod req {
    use crate::{
        borrow::{Borrowable, Ownable, Reborrowable},
        maybe::Maybe,
    };

    #[allow(missing_docs)]
    mod r#impl {
        use super::*;

        pub trait MOwnableRef<'a, T>: Maybe {
            /// Clone the inner reference and forward to [`Ownable::own`]
            fn to_owned(&self) -> T
            where
                Self: Clone;
            /// Forward to [`Ownable::own`]
            fn own(self) -> T;
        }

        impl<'a, T, MX> MOwnableRef<'a, T> for MX
        where
            MX: Maybe,
            MX::Inner: Ownable<'a, T>,
            T: Borrowable<'a, MX::Inner>,
        {
            fn to_owned(&self) -> T
            where
                Self: Clone,
            {
                self.clone().unwrap().own()
            }

            fn own(self) -> T {
                self.unwrap().own()
            }
        }

        pub trait MBorrowable<'a, R: 'a>: Maybe {
            /// Forward to [`Borrowable::borrow`]
            fn borrow(&'a self) -> R;
        }

        impl<'a, R: 'a, MX> MBorrowable<'a, R> for MX
        where
            MX: Maybe,
            MX::Inner: Borrowable<'a, R>,
        {
            fn borrow(&'a self) -> R {
                self.inner().borrow()
            }
        }

        pub trait MReborrowable<'a, R: 'a>: Maybe {
            /// Forward to [`Reborrowable::reborrow`]
            fn reborrow(self) -> R;
        }

        impl<'a, R: 'a, MX> MReborrowable<'a, R> for MX
        where
            MX: Maybe,
            MX::Inner: Reborrowable<'a, R>,
        {
            fn reborrow(self) -> R {
                self.unwrap().reborrow()
            }
        }

        pub trait MUnwrapInto<T>: Maybe {
            /// Forward to [`Into::into`]
            fn unwrap_into(self) -> T;
        }

        impl<T, MX> MUnwrapInto<T> for MX
        where
            MX: Maybe,
            MX::Inner: Into<T>,
        {
            fn unwrap_into(self) -> T {
                self.unwrap().into()
            }
        }
    }

    /// A [`Maybe`] whose inner value  is [`Ownable`]
    pub trait MOwnableRef<'a, T>: r#impl::MOwnableRef<'a, T> {}

    impl<'a, T, MX> MOwnableRef<'a, T> for MX
    where
        MX: Maybe,
        MX::Inner: Ownable<'a, T>,
        T: Borrowable<'a, MX::Inner>,
    {
    }

    /// A [`Maybe`] whose inner value is [`Borrowable`]
    pub trait MBorrowable<'a, R: 'a>: r#impl::MBorrowable<'a, R> {}

    impl<'a, R: 'a, MX> MBorrowable<'a, R> for MX
    where
        MX: Maybe,
        MX::Inner: Borrowable<'a, R>,
    {
    }

    /// A [`Maybe`] whose inner value is [`Reborrowable`]
    pub trait MReborrowable<'a, R: 'a>: r#impl::MReborrowable<'a, R> {}

    impl<'a, R: 'a, MX> MReborrowable<'a, R> for MX
    where
        MX: Maybe,
        MX::Inner: Reborrowable<'a, R>,
    {
    }

    /// A [`Maybe`] whose inner value is [`Into<T>`]
    pub trait MUnwrapInto<T>: r#impl::MUnwrapInto<T> {}

    impl<T, MX> MUnwrapInto<T> for MX
    where
        MX: Maybe,
        MX::Inner: Into<T>,
    {
    }
}
