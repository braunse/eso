// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;

/// Functions to ask about the status of an [`Eso`].
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
}
