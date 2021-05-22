// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Shorthand traits for requirements on [`Eso`]s
//! to keep the `where` clauses short and more readable.

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
            self.inner().to_owned()
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
