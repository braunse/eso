// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Shorthand traits for requirements on [`Maybe`]s
//! to keep the `where` clauses short and more readable.

use crate::{
    borrow::{Borrow, Take},
    maybe::Maybe,
};

#[allow(missing_docs)]
mod r#impl {
    use super::*;

    pub trait MTake<T>: Maybe {
        /// Clone the inner reference and forward to [`Take::own`]
        fn to_owned(&self) -> T
        where
            Self: Clone;
        /// Forward to [`Take::own`]
        fn own(self) -> T;
    }

    impl<T, MX> MTake<T> for MX
    where
        MX: Maybe,
        MX::Inner: Take<T>,
    {
        fn to_owned(&self) -> T
        where
            Self: Clone,
        {
            self.inner().to_owned()
        }

        fn own(self) -> T {
            self.unwrap().own()
        }
    }

    pub trait MBorrow<'a, R: 'a>: Maybe {
        /// Forward to [`Borrow::borrow`]
        fn borrow(&'a self) -> R;
    }

    impl<'a, R: 'a, MX> MBorrow<'a, R> for MX
    where
        MX: Maybe,
        MX::Inner: Borrow<'a, R>,
    {
        fn borrow(&'a self) -> R {
            self.inner().borrow()
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

/// A [`Maybe`] whose inner value  is [`Take`]
pub trait MTake<T>: r#impl::MTake<T> {}

impl<T, MX> MTake<T> for MX
where
    MX: Maybe,
    MX::Inner: Take<T>,
{
}

/// A [`Maybe`] whose inner value is [`Borrow`]
pub trait MBorrow<'a, R: 'a>: r#impl::MBorrow<'a, R> {}

impl<'a, R: 'a, MX> MBorrow<'a, R> for MX
where
    MX: Maybe,
    MX::Inner: Borrow<'a, R>,
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
