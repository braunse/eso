// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    eso::{
        req::{MBorrow, MIntern, MInternRef, MTake, MTryIntern, MTryInternRef},
        Eso,
    },
    maybe::{An, Maybe},
    shorthand::x,
};

/// Methods to transform an [`Eso`] between its different states.
impl<ME, MS, MO> Eso<ME, MS, MO> {
    /// Transform this [`Eso`] into one that can only be a static/shared
    /// reference or an owned value.
    ///
    /// This clones an ephemeral reference into an owned value via
    /// [`Take`](crate::borrow::Take)
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
        ME: MTake<MO::Inner>,
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
    /// assert_eq!(my_fn("Hello World").get_ref::<&str>(), "Hello World");
    /// ```
    ///
    /// The `to_static` method clones an ephemeral reference into an owned value via
    /// [`Take::own`](crate::borrow::Take::to_owned)
    /// but clones a shared/static reference or an owned value into the
    /// result unchanged.
    ///
    /// See [`Eso::into_static`] for considerations regarding the
    /// lifetime of the result.
    pub fn to_static(&self) -> x::sO<ME, MS, MO>
    where
        ME: MTake<MO::Inner> + Clone,
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
    /// [`Take`](crate::borrow::Take),
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
        ME: MTake<MO::Inner>,
        MS: MTake<MO::Inner>,
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
    /// [`Take`](crate::borrow::Take),
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
        ME: MTake<MO::Inner> + Clone,
        MS: MTake<MO::Inner> + Clone,
        MO: Maybe + Clone,
    {
        match self {
            Eso::E(e) => Eso::O(An(e.to_owned())),
            Eso::S(s) => Eso::O(An(s.to_owned())),
            Eso::O(o) => Eso::O(An(o.clone().unwrap())),
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
        MO: MBorrow<'a, ME::Inner>,
    {
        match self {
            Eso::E(e) => Eso::E(An(e.clone().unwrap())),
            Eso::S(s) => Eso::S(An(s.clone().unwrap())),
            Eso::O(o) => Eso::E(An(o.borrow())),
        }
    }

    /// Borrow an ephemeral reference.
    ///
    /// Clones an already-existing ephemeral reference, and
    /// [borrows](crate::borrow::Borrow::borrow) from a static reference
    /// or an owned value.
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
        MS: MBorrow<'a, ME::Inner>,
        MO: MBorrow<'a, ME::Inner>,
    {
        match self {
            Eso::E(e) => Eso::E(An(e.clone().unwrap())),
            Eso::S(s) => Eso::E(An(s.borrow())),
            Eso::O(o) => Eso::E(An(o.borrow())),
        }
    }

    /// Try transforming an ephemeral reference into a shared/static
    /// reference by [`interning`](crate::borrow::TryInternRef::try_intern_ref).
    ///
    /// ```
    /// # use eso::shorthand::t; use std::rc::Rc;
    /// type Str<'a> = t::ESO<&'a str, Rc<str>, String>;
    /// let ephemeral = Str::from_ref("Hello World");
    /// let shared = ephemeral.try_intern_ephemeral().expect("Should have worked");
    /// assert!(shared.is_static());
    /// assert_eq!(shared.get_ref::<&str>(), "Hello World");
    /// ```
    pub fn try_intern_ephemeral(self) -> Result<x::S<ME, MS, MO>, x::eo<ME, MS, MO>>
    where
        ME: MTryInternRef<MS::Inner>,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => match e.try_intern_ref() {
                Some(interned) => Ok(Eso::S(An(interned))),
                None => Err(Eso::E(e)),
            },
            Eso::S(s) => Ok(Eso::S(An(s.unwrap()))),
            Eso::O(o) => Err(Eso::O(o)),
        }
    }

    /// Try transforming an ephemeral reference or an owned value into a
    /// shared/static reference by [`interning`](crate::borrow::TryInternRef::try_intern_ref).
    ///
    /// ```
    /// # use eso::shorthand::t; use std::rc::Rc;
    /// type Str<'a> = t::ESO<&'a str, Rc<str>, String>;
    /// let owned = Str::from_owned("Hello World".to_string());
    /// let shared = owned.try_intern().expect("Should have worked");
    /// assert!(shared.is_static());
    /// assert_eq!(shared.get_ref::<&str>(), "Hello World");
    /// ```
    pub fn try_intern(self) -> Result<x::S<ME, MS, MO>, x::eo<ME, MS, MO>>
    where
        ME: MTryInternRef<MS::Inner>,
        MS: Maybe,
        MO: MTryIntern<MS::Inner>,
    {
        match self {
            Eso::E(e) => match e.try_intern_ref() {
                Some(interned) => Ok(Eso::S(An(interned))),
                None => Err(Eso::E(e)),
            },
            Eso::S(s) => Ok(Eso::S(An(s.unwrap()))),
            Eso::O(o) => match o.try_intern() {
                Ok(interned) => Ok(Eso::S(An(interned))),
                Err(o) => Err(Eso::O(o)),
            },
        }
    }

    /// Try transforming an ephemeral reference or an owned value into a
    /// shared/static reference by [`interning`](crate::borrow::TryInternRef::try_intern_ref),
    /// and if this does not work, clone it into an owned value via
    /// [`Take::own`](crate::borrow::Take::own).
    ///
    /// ```
    /// # use eso::shorthand::t; use std::rc::Rc;
    /// type Str<'a> = t::ESO<&'a str, Rc<str>, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// let interned = my_ref.intern_or_take();
    /// assert!(interned.is_static());
    /// assert_eq!(interned.get_ref::<&str>(), "Hello World");
    /// // TODO an example for failed interning
    /// ```

    pub fn intern_or_take(self) -> x::SO<ME, MS, MO>
    where
        ME: MTryInternRef<MS::Inner> + MTake<MO::Inner>,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => match e.try_intern_ref() {
                Some(interned) => Eso::S(An(interned)),
                None => Eso::O(An(e.own())),
            },
            Eso::S(s) => Eso::S(An(s.unwrap())),
            Eso::O(o) => Eso::O(An(o.unwrap())),
        }
    }

    /// Try transforming an ephemeral reference into a shared/static
    /// reference by [`interning`](crate::borrow::InternRef::intern_ref).
    ///
    /// ```
    /// # use eso::shorthand::t; use std::rc::Rc;
    /// type Str<'a> = t::ESO<&'a str, Rc<str>, String>;
    /// let ephemeral = Str::from_ref("Hello World");
    /// let shared = ephemeral.intern_ephemeral();
    /// assert!(shared.is_static());
    /// assert_eq!(shared.get_ref::<&str>(), "Hello World");
    /// ```
    pub fn intern_ephemeral(self) -> x::SO<ME, MS, MO>
    where
        ME: MInternRef<MS::Inner>,
        MS: Maybe,
        MO: Maybe,
    {
        match self {
            Eso::E(e) => Eso::S(An(e.intern_ref())),
            Eso::S(s) => Eso::S(An(s.unwrap())),
            Eso::O(o) => Eso::O(An(o.unwrap())),
        }
    }

    /// Try transforming an ephemeral reference or an owned value into a
    /// shared/static reference by [`interning`](crate::borrow::InternRef::intern_ref).
    ///
    /// ```
    /// # use eso::shorthand::t; use std::rc::Rc;
    /// type Str<'a> = t::ESO<&'a str, Rc<str>, String>;
    /// let owned = Str::from_owned("Hello World".to_string());
    /// let shared = owned.intern();
    /// assert!(shared.is_static());
    /// assert_eq!(shared.get_ref::<&str>(), "Hello World");
    /// ```
    pub fn intern(self) -> x::S<ME, MS, MO>
    where
        ME: MInternRef<MS::Inner>,
        MS: Maybe,
        MO: MIntern<MS::Inner>,
    {
        match self {
            Eso::E(e) => Eso::S(An(e.intern_ref())),
            Eso::S(s) => Eso::S(An(s.unwrap())),
            Eso::O(o) => Eso::S(An(o.intern())),
        }
    }
}
