# Eso - type-level building block for making `Cow`-like containers

[![Crates.io](https://img.shields.io/crates/v/eso)](https://crates.io/crates/eso)
[![docs.rs](https://img.shields.io/docsrs/eso)](https://docs.rs/eso)
[![GitHub issues](https://img.shields.io/github/issues/braunse/eso)](https://github.com/braunse/eso/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/braunse/eso)](https://github.com/braunse/eso/pulls)
![GitHub last commit](https://img.shields.io/github/last-commit/braunse/eso)
![Crates.io](https://img.shields.io/crates/l/eso)

This library provids the `Eso` struct, a versatile building block for making
newtypes that may own or reference their contents.

## How to use

Add to your `Cargo.toml` like:

```toml
[dependencies]
eso = "0.0.0"
```

## Example

Here is how to make a basic `Cow`-like type:

```rust
use eso::t;

pub struct SmartString<'a>(t::SO<&'a str, &'a str, String>);

impl SmartString {
    fn from_ref(c: &'static str) -> Self {
        SmartString(t::SO::from_static(c))
    }

    fn from_string(s: String) -> Self {
        SmartString(t::SO::from_owned(s))
    }

    fn into_owned(self) -> String {
        self.0.into_owning().safe_unwrap_owned()
    }

    fn is_owned(&self) -> bool {
        self.0.is_owning()
    }

    fn is_borrowed(&self) -> bool {
        self.0.is_reference()
    }

    fn to_mut(&mut self) -> &mut String {
        self.0.to_mut()
    }
}

impl Deref for SmartString {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.get_ref()
    }
}
```

## Details

`Eso` is _very_ flexible, because it is meant as a building block for library
authors who will restrict its flexibility to make sense for their respective
use cases:

- The `Eso` type itself can represent a choice out of any subset of

  - a borrowed reference
  - a static or shared reference
  - an owned value

  Which of these variants exist in the `Eso` type depends on the type parameters
  and can vary between usages in the client code.

- `Eso` generalizes references and ownership.

  For example, you can make a `Cow`-like type that stores a custom type instead
  of a normal reference, so you could make a copy-on-write
  [`OwningRef`](https://crates.io/crates/owning_ref)

The price for this flexibility is ergonomics.
When using `eso` the types can get rather long and the `where`-clauses in the
library are rather unwieldy.

## To Do

- More API docs
- More tests
- More examples
