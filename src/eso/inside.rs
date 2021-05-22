// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    eso::req::{MBorrowable, MOwnableRef, MReborrowable, MUnwrapInto},
    maybe::{An, Impossible, Maybe, No},
};

use super::*;

use std::borrow::Cow;

impl<ME, MS, O> Eso<ME, MS, An<O>> {
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

impl<ME, MS, MO> Eso<ME, MS, MO> {
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

    /// Transform into a [`Cow`].
    ///
    /// [Reborrows](crate::borrow::Reborrowable::reborrow) an ephemeral or
    /// static/shared reference, preserves an owned value.
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
