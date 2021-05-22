// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    maybe::{An, Impossible, Maybe, No, Relax},
    shorthand::t,
    unify::Unify3,
};

use super::*;

impl<ME, MS, MO> Eso<ME, MS, MO> {
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
