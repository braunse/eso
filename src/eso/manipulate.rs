// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::maybe::{Maybe, MaybeMap};

use super::*;

impl<ME, MS, MO> Eso<ME, MS, MO> {
    /// Transform `self´ by applying a function to the `E`
    /// variant while preserving the other variants.
    ///
    /// ```
    /// # use eso::shorthand::t;
    /// type Str<'a> = t::ESO<&'a str, &'static str, String>;
    /// let my_ref = Str::from_ref("Hello World");
    /// let mapped = my_ref.map_e(|_| "Ha!");
    /// assert_eq!(mapped.get_ref::<&str>(), "Ha!");
    ///
    /// let my_static = Str::from_static("Hello World");
    /// let mapped = my_static.map_e(|_| "Ha!");
    /// assert_eq!(mapped.get_ref::<&str>(), "Hello World");
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
    /// assert_eq!(mapped.get_ref::<&str>(), "Ha!");
    ///
    /// let my_ref = Str::from_ref("Hello World");
    /// let mapped = my_ref.map_s(|_| "Ha!");
    /// assert_eq!(mapped.get_ref::<&str>(), "Hello World");
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
    /// assert_eq!(mapped.get_ref::<&str>(), "HELLO WORLD");
    ///
    /// let my_static = Str::from_static("Hello World");
    /// let mapped = my_static.map_o(|e| e.to_uppercase());
    /// assert_eq!(mapped.get_ref::<&str>(), "Hello World");
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
    ///     let the_string = mapped.get_ref::<&str>();
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
}
