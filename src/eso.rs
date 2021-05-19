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
    shorthand::{t, x},
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
    ///
    /// ```
    /// # use ::eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn make_a_str<'a>(s: &'a str) -> Str<'a> {
    ///     Str::from_ref(s)
    /// }
    /// let my_string = String::from("Hello World");
    /// let my_str = make_a_str(my_string.as_str());
    /// ```

    pub const fn from_ref(e: E) -> Self {
        Eso::E(An(e))
    }
}

impl<ME, S, MO> Eso<ME, An<S>, MO> {
    /// Create an [`Eso`] from a shared/static reference
    ///
    /// ```
    /// # use ::eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_str = Str::from_static("Hello World");
    /// ```
    pub const fn from_static(s: S) -> Self {
        Eso::S(An(s))
    }
}

impl<ME, MS, O> Eso<ME, MS, An<O>> {
    /// Create an [`Eso`] from an owned object
    ///
    /// ```
    /// # use ::eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_str = Str::from_owned("Hello World".into());
    /// ```
    pub const fn from_owned(o: O) -> Self {
        Eso::O(An(o))
    }

    /// Get a mutable reference to the contained owned value,
    /// cloning out of a referenced object if necessary.
    ///
    /// ```
    /// # use ::eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let mut my_str = Str::from_ref("Hello ");
    /// my_str.to_mut().push_str("World!");
    /// assert!(my_str.is_owning());
    /// assert_eq!(my_str.get_ref(), "Hello World!");
    /// ```
    pub fn to_mut(&mut self) -> &mut O
    where
        ME: MOwnableRef<O> + Clone,
        MS: MOwnableRef<O> + Clone,
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

    /// Mutate the owned value. If no owned value is contained,
    /// the referenced value will be cloned into an owned form.
    ///
    /// ```
    /// # use ::eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let mut my_str = Str::from_ref("Hello ");
    /// my_str.mutate(|s| s.push_str("World!"));
    /// assert!(my_str.is_owning());
    /// assert_eq!(my_str.get_ref(), "Hello World!");
    /// ```
    pub fn mutate<F, T>(&mut self, f: F) -> T
    where
        ME: MOwnableRef<O> + Clone,
        MS: MOwnableRef<O> + Clone,
        F: FnOnce(&mut O) -> T,
    {
        match self {
            Eso::E(e) => *self = Eso::O(An(e.to_owned())),
            Eso::S(s) => *self = Eso::O(An(s.to_owned())),
            Eso::O(_) => (),
        };
        match self {
            Eso::O(An(o)) => f(o),
            _ => unreachable!(),
        }
    }
}

impl<E, MS, O> Eso<An<E>, MS, An<O>> {
    /// Create an [`Eso`] from a [`Cow`].
    ///
    /// This will either convert the reference from a [`Cow::Borrowed`]
    /// into the correct generalized reference type via [`Reborrowable`]
    /// or take ownership of the owned value from a [`Cow::Owned`].
    ///
    /// ```
    /// # use eso::shorthand::t; use std::borrow::Cow;
    /// type Str<'a> = t::EO<&'a str, &'static str, String>;
    ///
    /// let owned_eso = Str::from_cow(Cow::Owned("Hello World".to_string()));
    /// assert!(owned_eso.is_owning());
    /// assert_eq!(owned_eso.get_ref(), "Hello World");
    ///
    /// let borrowed_eso = Str::from_cow(Cow::Borrowed("Hello World"));
    /// assert!(borrowed_eso.is_reference());
    /// assert_eq!(borrowed_eso.get_ref(), "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_function(a_borrowed_str: &str) {
    ///     let ephemeral = Str::from_ref(a_borrowed_str);
    ///     let shared = Str::from_static("Hello World");
    ///     let owned = Str::from_owned("Hello World".to_string());
    ///     assert!(ephemeral.is_ephemeral());
    ///     assert!(!shared.is_ephemeral());
    ///     assert!(!owned.is_ephemeral());
    /// }
    /// my_function("Hello World");
    /// ```
    pub fn is_ephemeral(&self) -> bool {
        matches!(self, Eso::E(_))
    }

    /// Returns `true` if the [`Eso`] is of the [`Eso::S`] variant.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_function(a_borrowed_str: &str) {
    ///     let ephemeral = Str::from_ref(a_borrowed_str);
    ///     let shared = Str::from_static("Hello World");
    ///     let owned = Str::from_owned("Hello World".to_string());
    ///     assert!(!ephemeral.is_static());
    ///     assert!(shared.is_static());
    ///     assert!(!owned.is_static());
    /// }
    /// my_function("Hello World");
    /// ```
    pub fn is_static(&self) -> bool {
        matches!(self, Eso::S(_))
    }

    /// Returns `true` if the [`Eso`] is of the [`Eso::O`] variant.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_function(a_borrowed_str: &str) {
    ///     let ephemeral = Str::from_ref(a_borrowed_str);
    ///     let shared = Str::from_static("Hello World");
    ///     let owned = Str::from_owned("Hello World".to_string());
    ///     assert!(!ephemeral.is_owning());
    ///     assert!(!shared.is_owning());
    ///     assert!(owned.is_owning());
    /// }
    /// my_function("Hello World");
    /// ```
    pub fn is_owning(&self) -> bool {
        matches!(self, Eso::O(_))
    }

    /// Returns `true` if the [`Eso`] does not own the contained value,
    /// i. e. it is of the [`Eso::E`] or [`Eso::S`] variants.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_function(a_borrowed_str: &str) {
    ///     let ephemeral = Str::from_ref(a_borrowed_str);
    ///     let shared = Str::from_static("Hello World");
    ///     let owned = Str::from_owned("Hello World".to_string());
    ///     assert!(ephemeral.is_reference());
    ///     assert!(shared.is_reference());
    ///     assert!(!owned.is_reference());
    /// }
    /// my_function("Hello World");
    /// ```
    pub fn is_reference(&self) -> bool {
        !matches!(self, Eso::O(_))
    }

    /// Returns `true` if the [`Eso`] is lasting, i. e. it is not
    /// an ephemeral reference.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_function(a_borrowed_str: &str) {
    ///     let ephemeral = Str::from_ref(a_borrowed_str);
    ///     let shared = Str::from_static("Hello World");
    ///     let owned = Str::from_owned("Hello World".to_string());
    ///     assert!(!ephemeral.is_lasting());
    ///     assert!(shared.is_lasting());
    ///     assert!(owned.is_lasting());
    /// }
    /// my_function("Hello World");
    /// ```
    pub fn is_lasting(&self) -> bool {
        !matches!(self, Eso::E(_))
    }

    /// Transform this [`Eso`] into one that can only be a static/shared
    /// reference or an owned value.
    ///
    /// This clones an ephemeral reference into an owned value via
    /// [`Ownable`](crate::borrow::Ownable)
    /// but will move a shared/static reference or an owned value into the
    /// result unchanged.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// type StaticStr<'a> = t::SO<&'a str, &'static str, String>;
    /// let my_reference = Str::from_ref("Hello World");
    /// assert!(my_reference.is_ephemeral());
    /// let my_static: StaticStr = my_reference.into_static();
    /// assert!(my_static.is_lasting());
    /// ```
    ///
    /// The conversion will not remove the lifetimes from the type of the
    /// reference:
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// # fn function_consuming_static<T: 'static>(_: T) {}
    /// # type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_fn(borrowed: &str) {
    ///     let my_reference = Str::from_ref(borrowed);
    ///     let my_static = my_reference.into_static();
    ///     function_consuming_static(my_static);
    /// }
    /// my_fn("Hello World");
    /// ```
    ///
    /// However, given that there is a type-level proof that the return value
    /// of this function cannot be of the `E` variant, the [`relax`](Eso::relax)
    /// function can be used to drop the `'a` lifetime:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// # fn function_consuming_static<T: 'static>(_: T) {}
    /// # type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_fn(borrowed: &str) {
    ///     let my_reference = Str::from_ref(borrowed);
    ///     let my_static: Str<'static> = my_reference.into_static().relax();
    ///     function_consuming_static(my_static);
    /// }
    /// my_fn("Hello World");
    /// ```
    pub fn into_static(self) -> x::sO<ME, MS, MO>
    where
        ME: MOwnableRef<MO::Inner>,
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
    /// ```
    /// # use eso::shorthand::t;
    /// # fn function_consuming_static<T: 'static>(_: T) {}
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn my_fn(borrowed: &str) -> Str {
    ///     let my_reference = Str::from_ref(borrowed);
    ///     let my_static: Str<'static> = my_reference.to_static().relax();
    ///     function_consuming_static(my_static);
    ///     my_reference
    /// }
    /// assert_eq!(my_fn("Hello World").get_ref(), "Hello World");
    /// ```
    ///
    /// The `to_static` method clones an ephemeral reference into an owned value via
    /// [`Ownable::to_owned`](crate::borrow::Ownable::to_owned)
    /// but clones a shared/static reference or an owned value into the
    /// result unchanged.
    /// This may be preferable to calling [`clone`](Clone::clone) and
    /// [`to_static`](Eso::to_static) in case one of the contained
    /// types has an optimized implementation of [`Ownable::to_owned`].
    ///
    /// See [`Eso::into_static`] for considerations regarding the
    /// lifetime of the result.
    pub fn to_static(&self) -> x::sO<ME, MS, MO>
    where
        ME: MOwnableRef<MO::Inner> + Clone,
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_str = Str::from_ref("Hello World");
    /// let my_owned = my_str.into_owning();
    /// assert!(my_owned.is_owning());
    /// ```
    pub fn into_owning(self) -> x::O<ME, MS, MO>
    where
        ME: MOwnableRef<MO::Inner>,
        MS: MOwnableRef<MO::Inner>,
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_str = Str::from_ref("Hello World");
    /// let my_owned = my_str.to_owning();
    /// assert!(my_str.is_ephemeral()); // <-- my_str is still alive
    /// assert!(my_owned.is_owning());
    /// ```
    pub fn to_owning(&self) -> x::O<ME, MS, MO>
    where
        ME: MOwnableRef<MO::Inner> + Clone,
        MS: MOwnableRef<MO::Inner> + Clone,
        MO: Maybe + Clone,
    {
        match self {
            Eso::E(e) => Eso::O(An(e.to_owned())),
            Eso::S(s) => Eso::O(An(s.to_owned())),
            Eso::O(o) => Eso::O(An(o.clone().unwrap())),
        }
    }

    /// Borrow a generalized reference of type `T` from this [`Eso`].
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let ephemeral = Str::from_ref("Hello World");
    /// let owning = Str::from_owned("Hello World".to_string());
    /// assert_eq!(ephemeral.get_ref(), owning.get_ref());
    /// ```
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
    /// if it actually contains an owned value:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Int<'a> = t::ESO<&'a i32, &'static i32, i32>;
    /// let mut my_int = Int::from_owned(40);
    /// if let Some(mut_ref) = my_int.try_get_mut() {
    ///     *mut_ref += 2;
    /// }
    /// assert_eq!(my_int.get_ref(), &42);
    /// ```
    ///
    /// Return `None` if `self` contains a reference:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Int<'a> = t::ESO<&'a i32, &'static i32, i32>;
    /// let mut my_int = Int::from_ref(&42);
    /// assert!(matches!(my_int.try_get_mut(), None));
    /// ```
    ///
    /// If clone-on-write behavior is desired, use the
    /// [`get_mut`](Eso::get_mut) method instead.
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// let ephemeral = my_ref.try_split_ephemeral().unwrap();
    /// //  ^^^^^^^^^---- carries type-level prrof that it can only be an Eso::E!
    /// let the_ref = ephemeral.safe_unwrap_ephemeral();
    /// assert_eq!(the_ref, "Hello World");
    /// ```
    ///
    /// Otherwise, casts the type parameters to prove the non-match:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::EO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// let owned = my_ref.try_split_ephemeral().unwrap_err();
    /// //  ^^^^^---- carries type-level prrof that it can not be an Eso::E.
    /// //            Since we used the t::EO alias, that only leaves Eso::O.
    /// let the_string: String = owned.safe_unwrap_owned();
    /// assert_eq!(&the_string, "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_static("Hello World");
    /// let shared = my_ref.try_split_static().unwrap();
    /// //  ^^^^^^^^^---- carries type-level prrof that it can only be an Eso::E!
    /// let the_ref = shared.safe_unwrap_static();
    /// assert_eq!(the_ref, "Hello World");
    /// ```
    ///
    /// Otherwise, casts the type parameters to prove the non-match:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::SO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// let owned = my_ref.try_split_static().unwrap_err();
    /// //  ^^^^^---- carries type-level prrof that it can not be an Eso::E.
    /// //            Since we used the t::EO alias, that only leaves Eso::O.
    /// let the_string: String = owned.safe_unwrap_owned();
    /// assert_eq!(&the_string, "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// let owned = my_ref.try_split_owned().unwrap();
    /// //  ^^^^^---- carries type-level prrof that it can only be an Eso::E!
    /// let the_string = owned.safe_unwrap_owned();
    /// assert_eq!(&the_string, "Hello World");
    /// ```
    ///
    /// Otherwise, casts the type parameters to prove the non-match:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::SO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_static("Hello World");
    /// let shared = my_ref.try_split_owned().unwrap_err();
    /// //  ^^^^^^---- carries type-level prrof that it can not be an Eso::E.
    /// //             Since we used the t::EO alias, that only leaves Eso::O.
    /// let the_str = shared.safe_unwrap_static();
    /// assert_eq!(the_str, "Hello World");
    /// ```
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

    /// Retrieve the contained ephemeral reference, if `self` is [`Eso::E`].
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// assert_eq!(
    ///     my_ref.try_unwrap_ephemeral().unwrap(),
    ///     "Hello World"
    /// );
    /// ```
    ///
    /// Otherwise, casts the type parameters to prove the non-match:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::EO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// let owned = my_ref.try_unwrap_ephemeral().unwrap_err();
    /// //  ^^^^^---- carries type-level prrof that it can not be an Eso::E.
    /// //            Since we used the t::EO alias, that only leaves Eso::O.
    /// let the_string: String = owned.safe_unwrap_owned();
    /// assert_eq!(&the_string, "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// assert_eq!(
    ///     my_ref.try_unwrap_owned().unwrap(),
    ///     "Hello World".to_string()
    /// );
    /// ```
    ///
    /// Otherwise, casts the type parameters to prove the non-match:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::EO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// let ephemeral = my_ref.try_unwrap_owned().unwrap_err();
    /// //  ^^^^^^^^^---- carries type-level prrof that it can not be an Eso::E.
    /// //                Since we used the t::EO alias, that only leaves Eso::O.
    /// let the_str = ephemeral.safe_unwrap_ephemeral();
    /// assert_eq!(the_str, "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_static("Hello World");
    /// assert_eq!(
    ///     my_ref.try_unwrap_static().unwrap(),
    ///     "Hello World"
    /// );
    /// ```
    ///
    /// Otherwise, casts the type parameters to prove the non-match:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::SO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// let owned = my_ref.try_unwrap_static().unwrap_err();
    /// //  ^^^^^---- carries type-level prrof that it can not be an Eso::E.
    /// //            Since we used the t::SO alias, that only leaves Eso::O.
    /// let the_string = owned.safe_unwrap_owned();
    /// assert_eq!(&the_string, "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_shared = Str::from_static("Hello World");
    /// let my_owned = Str::from_owned("Hello World".to_string());
    /// assert!(my_shared.reference().is_static());
    /// assert!(my_owned.reference().is_ephemeral());
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_shared = Str::from_static("Hello World");
    /// let my_owned = Str::from_owned("Hello World".to_string());
    /// assert!(my_shared.ephemeral().is_ephemeral());
    /// assert!(my_owned.ephemeral().is_ephemeral());
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t; use std::borrow::Cow;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// let my_owned = Str::from_owned("Hello World".to_string());
    /// assert!(matches!(my_ref.into_cow(), Cow::Borrowed("Hello World")));
    /// assert!(matches!(my_owned.into_cow(), Cow::Owned(_)));
    /// ```
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
    ///
    /// An [`Eso`] always carries all three of its type parameters
    /// in its type signature. Sometimes this leads to errors:
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::SO<&'a str, &'static str, String>;
    /// # fn function_that_expects_a_static(s: Str<'static>) {}
    /// fn tie_to_lifetime<'a>(lifetime_of: &'a i32) -> Str<'a> {
    ///     Str::from_static("Hello World")
    /// }
    /// let i = 42;
    /// let my_str = tie_to_lifetime(&i); // <-- tied to i's lifetime
    /// function_that_expects_a_static(my_str); // <-- ERROR
    /// ```
    ///
    /// Since the [`t::SO`] alias contains type-level proof that
    /// the [`E`](Eso::E) case is not possible, the first type
    /// parameter can be safely changed from `&'a str` to
    /// `&'static str`:
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// # type Str<'a> = t::SO<&'a str, &'static str, String>;
    /// # fn function_that_expects_a_static(s: Str<'static>) {}
    /// # fn tie_to_lifetime<'a>(lifetime_of: &'a i32) -> Str<'a> {
    /// #     Str::from_static("Hello World")
    /// # }
    /// let i = 42;
    /// let my_str = tie_to_lifetime(&i); // <-- tied to i's lifetime
    /// let my_str: Str<'static> = my_str.relax(); // <-- known to be static
    /// function_that_expects_a_static(my_str); // No Error
    /// ```
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
    /// an expected configuration. See [`Eso::relax`] for
    /// more information.
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

    /// Transform `self´ by applying a function to the `E`
    /// variant while preserving the other variants.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// let mapped = my_ref.map_e(|_| "Ha!");
    /// assert_eq!(mapped.get_ref(), "Ha!");
    ///
    /// let my_static = Str::from_static("Hello World");
    /// let mapped = my_static.map_e(|_| "Ha!");
    /// assert_eq!(mapped.get_ref(), "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_static = Str::from_static("Hello World");
    /// let mapped = my_static.map_s(|_| "Ha!");
    /// assert_eq!(mapped.get_ref(), "Ha!");
    ///
    /// let my_ref = Str::from_ref("Hello World");
    /// let mapped = my_ref.map_s(|_| "Ha!");
    /// assert_eq!(mapped.get_ref(), "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_owned("Hello World".to_string());
    /// let mapped = my_ref.map_o(|e| e.to_uppercase());
    /// assert_eq!(mapped.get_ref(), "HELLO WORLD");
    ///
    /// let my_static = Str::from_static("Hello World");
    /// let mapped = my_static.map_o(|e| e.to_uppercase());
    /// assert_eq!(mapped.get_ref(), "Hello World");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// fn check(s: Str, into: &str) {
    ///     let mapped = s.map(
    ///             |e| "Ephemeral",
    ///             |s| "Static",
    ///             |o| o.to_uppercase());
    ///     let the_string = mapped.get_ref();
    ///     assert_eq!(the_string, into);
    /// }
    /// check(Str::from_ref("Hello World"), "Ephemeral");
    /// check(Str::from_static("Hello World"), "Static");
    /// check(Str::from_owned("Hello World".to_string()), "HELLO WORLD");
    /// ```
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
    ///
    /// ```
    /// # use eso::shorthand::t; use std::rc::Rc;
    /// type Str = t::ESO<i32, char, String>;
    /// fn check(s: Str, should: &str) {
    ///     let str = s.merge_with(
    ///         |e| (format!("{}", e)),
    ///         |s| (format!("{}", s)),
    ///         |o| o);
    ///     assert_eq!(&str, should);
    /// }
    /// check(Str::from_ref(42), "42");
    /// check(Str::from_static('!'), "!");
    /// check(Str::from_owned("Hello World".to_owned()), "Hello World");
    /// ```
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
    /// # use ::eso::{eso::*, shorthand::*};
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
    /// # use ::eso::{eso::*, shorthand::*};
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
    /// ```
    /// # use eso::shorthand::t;
    /// type OnlyE<'a> = t::E<&'a str, &'static str, String>;
    /// let only_e = OnlyE::from_ref("Hello World");
    /// let unwrapped = only_e.safe_unwrap_ephemeral();
    /// assert_eq!(unwrapped, "Hello World");
    /// ```
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an ephemeral reference, and thus cannot fail.
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// type MaybeS<'a> = t::ES<&'a str, &'static str, String>;
    /// let maybe_s = MaybeS::from_ref("Hello World");
    /// // Compile error: maybe_s could also contain a static reference!
    /// let unwrapped = maybe_s.safe_unwrap_ephemeral();
    /// ```
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
    /// ```
    /// # use eso::shorthand::t;
    /// type OnlyO<'a> = t::O<&'a str, &'static str, String>;
    /// let only_o = OnlyO::from_owned("Hello World".to_string());
    /// let unwrapped = only_o.safe_unwrap_owned();
    /// assert_eq!(&unwrapped, "Hello World");
    /// ```
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an owned value, and thus cannot fail.
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// type MaybeS<'a> = t::SO<&'a str, &'static str, String>;
    /// let maybe_s = MaybeS::from_owned("Hello World".to_string());
    /// // Compile error: maybe_s could also contain a static reference!
    /// let unwrapped = maybe_s.safe_unwrap_owned();
    /// ```
    pub fn safe_unwrap_owned(self) -> O {
        match self {
            Eso::E(e) => e.absurd(),
            Eso::S(s) => s.absurd(),
            Eso::O(An(o)) => o,
        }
    }

    /// Safely reference the owned value contained in this [`Eso`].
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type OnlyO<'a> = t::O<&'a str, &'static str, String>;
    /// let only_o = OnlyO::from_owned("Hello World".to_string());
    /// assert!(only_o.get_owned_ref().capacity() >= 11);
    /// ```
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an ephemeral reference, and thus cannot fail.
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// type MaybeS<'a> = t::SO<&'a str, &'static str, String>;
    /// let maybe_s = MaybeS::from_owned("Hello World".to_string());
    /// maybe_s.get_owned_ref()
    /// ```
    pub fn get_owned_ref(&self) -> &O {
        match self {
            Eso::E(e) => e.absurd(),
            Eso::S(s) => s.absurd(),
            Eso::O(An(o)) => o,
        }
    }

    /// Safely and mutably reference the owned value contained in this [`Eso`].
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type OnlyO<'a> = t::O<&'a str, &'static str, String>;
    /// let mut only_o = OnlyO::from_owned("Hello ".to_string());
    /// only_o.get_mut().push_str("World");
    /// assert_eq!(only_o.get_ref(), "Hello World");
    /// ```
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain an ephemeral reference, and thus cannot fail.
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// type MaybeS<'a> = t::SO<&'a str, &'static str, String>;
    /// let maybe_s = MaybeS::from_owned("Hello World".to_string());
    /// maybe_s.get_mut()
    /// ```
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
    /// ```
    /// # use eso::shorthand::t;
    /// type OnlyS<'a> = t::S<&'a str, &'static str, String>;
    /// let only_s = OnlyS::from_static("Hello World");
    /// let unwrapped = only_s.safe_unwrap_static();
    /// assert_eq!(unwrapped, "Hello World");
    /// ```
    ///
    /// This method is only callable on an [`Eso`] that is statically proven
    /// to contain a static/shared reference, and thus cannot fail.
    ///
    /// ```compile_fail
    /// # use eso::shorthand::t;
    /// type MaybeE<'a> = t::ES<&'a str, &'static str, String>;
    /// let maybe_e = MaybeE::from_static("Hello World");
    /// // Compile error: maybe_s could also contain a static reference!
    /// let unwrapped = maybe_e.safe_unwrap_owned();
    /// ```
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

/// Shorthand traits for requirements on [`Eso`]s
/// to keep the `where` clauses short and more readable.
pub mod req {
    use crate::{
        borrow::{Borrowable, Ownable, Reborrowable},
        maybe::Maybe,
    };

    #[allow(missing_docs)]
    mod r#impl {
        use super::*;

        pub trait MOwnableRef<T>: Maybe {
            /// Clone the inner reference and forward to [`Ownable::own`]
            fn to_owned(&self) -> T
            where
                Self: Clone;
            /// Forward to [`Ownable::own`]
            fn own(self) -> T;
        }

        impl<T, MX> MOwnableRef<T> for MX
        where
            MX: Maybe,
            MX::Inner: Ownable<T>,
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
    pub trait MOwnableRef<T>: r#impl::MOwnableRef<T> {}

    impl<T, MX> MOwnableRef<T> for MX
    where
        MX: Maybe,
        MX::Inner: Ownable<T>,
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
