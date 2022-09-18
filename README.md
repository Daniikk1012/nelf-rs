# An implementation of a NELF parser written in Rust

NELF (No Escape List Format) is a human-readable file format for encoding lists
of arbitrary strings without escaping their contents. This file format has the
same use cases as CSV and allows programs to take out strings contained in a
NELF list by taking a pointer (A string slice, view) to the source string, which
removes the need to allocate additional space for them. The specification is yet
to be developed.

## Stability

The API for this cratee is unstable, if you want to use it, always specify the
minor version you want to use in your `Cargo.toml`.

There are plans to generalize the parser for string slices and to use [`serde`]
crate for easier serializing/deserizalizing in the future.

## Get started

The API docs are available [here](https://docs.rs/nelf)

## License

This library is licensed under Mozilla Public License, v. 2.0. The text of the
license is available [here](LICENSE).

[`serde`]: https://serde.rs
