// Copyright (c) 2021 Sebastien Braun
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env;

use eyre::Result;
use xshell::{pushenv, Pushenv};

pub fn amend_env_var(
    name: impl AsRef<str>,
    separator: impl AsRef<str>,
    value: impl AsRef<str>,
) -> Result<Pushenv> {
    let mut envvar = env::var(name.as_ref())
        .map(Some)
        .or_else(|e| match e {
            env::VarError::NotPresent => Ok(None),
            e => Err(e),
        })?
        .unwrap_or_default();

    // Add the separator, if the variable already exists and has a value
    if !envvar.is_empty() {
        envvar.push_str(separator.as_ref());
    }
    envvar.push_str(value.as_ref());
    Ok(pushenv(name.as_ref(), envvar))
}
