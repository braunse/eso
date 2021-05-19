// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Shorthand type aliases to refer to [`Eso`] with different combinations of
//! type parameters.
//!
//! Import the sub-modules [`t`], [`s`] or [`x`] qualified to use them:
//!
//! ```
//! # use ::eso::{eso, maybe::{An,No}, shorthand::t};
//! type MyString<'a> = t::ESO<&'a str, &'static str, String>;
//! type Expanded<'a> = eso::Eso<An<&'a str>, An<&'static str>, An<String>>;
//! let a_str: MyString<'_> = MyString::from_static("Hello World");
//! let a_str: Expanded<'_> = a_str;
//! ```

use crate::eso::Eso;

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

/// Shorthand type aliases for transformations of an [`Eso`],
/// where the `E` and `S` parameters are equal.
///
/// This can be used to specify types that are never ephemeral.
///
/// The type names derive from the two remaining components `MS` and `MO`,
/// but the input type arguments are transformed according to the
/// rules:
///
/// | Rule      | Type parameter  | Transformed to | Meaning      |
/// |-----------|-----------------|----------------|--------------|
/// | Uppercase | `MT`: [`Maybe`] | `An<T::Inner>` | present      |
/// | Missing   | `MT`: [`Maybe`] | `No<T::Inner>` | absent       |
/// | Lowercase | `MT`: [`Maybe`] | `MT`           | pass-through |
#[allow(non_camel_case_types)]
pub mod s {
    use super::t;
    use crate::maybe::Maybe;

    /// [`Eso`] with `E` = `S` present, `O` present, see [shorthand module docs](super::s)
    pub type SO<S, O> = t::SO<S, S, O>;

    /// [`Eso`] with `E` = `S` pass-through, `O` present, see [shorthand module docs](super::s)
    pub type sO<MS, O> = t::sO<<MS as Maybe>::Inner, MS, O>;

    /// [`Eso`] with `E` = `S` present, `O` pass-through, see [shorthand module docs](super::s)
    pub type So<S, MO> = t::So<S, S, MO>;

    /// [`Eso`] with `E` = `S` pass-through, `O` pass-through, see [shorthand module docs](super::s)
    pub type so<MS, MO> = t::so<<MS as Maybe>::Inner, MS, MO>;

    /// [`Eso`] with `E` = `S` present, `O` absent, see [shorthand module docs](super::s)
    pub type S<S, O> = t::S<S, S, O>;

    /// [`Eso`] with `E` = `S` absent, `O` present, see [shorthand module docs](super::s)
    pub type O<S, O> = t::O<S, S, O>;

    /// [`Eso`] with `E` = `S` pass-through, `O` absent, see [shorthand module docs](super::s)
    pub type s<MS, O> = t::s<<MS as Maybe>::Inner, MS, O>;

    /// [`Eso`] with `E` = `S` absent, `O` pass-through, see [shorthand module docs](super::s)
    pub type o<S, MO> = t::o<S, S, MO>;

    /// [`Eso`] with `E` = `S` absent, `O` absent, see [shorthand module docs](super::s)
    pub type None<S, O> = t::None<S, S, O>;
}
