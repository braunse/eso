// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[path = "../examples/simple_newtype.rs"]
mod simple_newtype;

use simple_newtype::*;

#[test]
fn test_1() {
    let a = WrappedString::from_str("Hello World");
    let b = WrappedString::from_static("Hello World");
    let c = WrappedString::from_string("Hello World".into());

    assert_eq!(&*a, &*b);
    assert_eq!(&*b, &*c);
    assert_eq!(&*a, &*c);
}
