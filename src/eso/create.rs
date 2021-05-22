// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::borrow::Cow;

use crate::{borrow::Reborrowable, maybe::An};

use super::*;

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
