//! # No Escape List Format parser library
//!
//! This library provides a simple implementation for NELF parser, a
//! human-readable textfile format used to represent lists of strings of bytes
//! without needing to escape any characters, which allows to borrow strings
//! from the string that represents the encoded list without the need to
//! allocate additional space.
//!
//! ## Stability
//!
//! The API for this crate is unstable, if you want to use it, always specify
//! the minor version you want to use in your `Cargo.toml`.
//! ## Get started
//!
//! To get started, check the documentation for the following structures and
//! traits:
//!
//! * [`NelfIter`]
//! * [`ToCell`]
//! * [`ToNelf`]
//!
//! [`NelfIter`]: NelfIter
//! [`ToCell`]: ToCell
//! [`ToNelf`]: ToNelf

#![deny(missing_docs)]

use private::{ToCellSealed, ToNelfSealed};

/// Iterator of cells contained in the encoded list.
///
/// Borrows the source and iterates of string slices borrowing from that source.
#[derive(Clone, Copy)]
pub struct NelfIter<'a> {
    string: &'a [u8],
    index: usize,
}

impl<'a> NelfIter<'a> {
    /// Construct the iterator borrowing from the encoded list.
    pub fn from_string(string: &'a [u8]) -> Self {
        NelfIter { string, index: 0 }
    }
}

impl<'a> Iterator for NelfIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let lch;
        (self.index, lch) = self.string[self.index..]
            .iter()
            .enumerate()
            .find(|&(_, &ch)| matches!(ch, b'|' | b'/' | b'\\'))
            .map(|(index, &ch)| (self.index + index, ch))?;

        let len = self.string[self.index..]
            .iter()
            .enumerate()
            .find(|&(_, &ch)| ch != lch)
            .map(|(index, _)| index)?;

        let start = self.index + len;

        self.index = start;

        let rch = match lch {
            b'|' => b'|',
            b'/' => b'\\',
            b'\\' => b'/',
            _ => unreachable!(),
        };

        let mut count = 0;

        while self.index < self.string.len() {
            if count == len {
                break;
            }

            if self.string[self.index] == rch {
                count += 1;
            } else {
                count = 0;
            }

            self.index += 1;
        }

        Some(if count == len {
            &self.string[start..self.index - len]
        } else {
            &self.string[start..]
        })
    }
}

/// Trait used to encode strings as cells in a NELF list.
///
/// Already implemented for the most commonly used types, sealed.
pub trait ToCell: ToCellSealed {
    /// Encodes the value as a NELF cell.
    fn to_cell(self) -> Vec<u8>;
}

impl ToCellSealed for &[u8] {}

impl ToCell for &[u8] {
    fn to_cell(self) -> Vec<u8> {
        let mut result = Vec::new();

        if !self.is_empty() {
            let mut pipe = true;
            let mut forward = true;
            let mut back = true;

            match self.first().unwrap() {
                b'|' => pipe = false,
                b'/' => forward = false,
                b'\\' => back = false,
                _ => (),
            }

            match self.last().unwrap() {
                b'|' => pipe = false,
                b'/' => back = false,
                b'\\' => forward = false,
                _ => (),
            }

            let mut pipe_max = usize::MAX;
            let mut forward_max = usize::MAX;
            let mut back_max = usize::MAX;

            if pipe {
                pipe_max = self
                    .iter()
                    .scan(0, |state, &x| {
                        if x == b'|' {
                            *state += 1;
                        } else {
                            *state = 0;
                        }

                        Some(*state)
                    })
                    .max()
                    .unwrap()
                    + 1;
            }

            if forward {
                forward_max = self
                    .iter()
                    .scan(0, |state, &x| {
                        if x == b'\\' {
                            *state += 1;
                        } else {
                            *state = 0;
                        }

                        Some(*state)
                    })
                    .max()
                    .unwrap()
                    + 1;
            }

            if back {
                back_max = self
                    .iter()
                    .scan(0, |state, &x| {
                        if x == b'/' {
                            *state += 1;
                        } else {
                            *state = 0;
                        }

                        Some(*state)
                    })
                    .max()
                    .unwrap()
                    + 1;
            }

            let min = pipe_max.min(forward_max).min(back_max);

            if pipe_max == min {
                result.resize(result.len() + min, b'|');
                result.extend_from_slice(self);
                result.resize(result.len() + min, b'|');
            } else if forward_max == min {
                result.resize(result.len() + min, b'/');
                result.extend_from_slice(self);
                result.resize(result.len() + min, b'\\');
            } else {
                result.resize(result.len() + min, b'\\');
                result.extend_from_slice(self);
                result.resize(result.len() + min, b'/');
            }
        } else {
            result.extend_from_slice(b"/\\");
        }

        result
    }
}

impl<const N: usize> ToCellSealed for &[u8; N] {}

impl<const N: usize> ToCell for &[u8; N] {
    fn to_cell(self) -> Vec<u8> {
        self.as_slice().to_cell()
    }
}

impl<const N: usize> ToCellSealed for [u8; N] {}

impl<const N: usize> ToCell for [u8; N] {
    fn to_cell(self) -> Vec<u8> {
        self.as_slice().to_cell()
    }
}

impl ToCellSealed for &Vec<u8> {}

impl ToCell for &Vec<u8> {
    fn to_cell(self) -> Vec<u8> {
        self.as_slice().to_cell()
    }
}

impl ToCellSealed for Vec<u8> {}

impl ToCell for Vec<u8> {
    fn to_cell(self) -> Vec<u8> {
        self.as_slice().to_cell()
    }
}

/// Trait used to convert containers of strings to NELF strings.
///
/// Already implemented for all iterables of byte slices. Sealed.
pub trait ToNelf: ToNelfSealed {
    /// Converts the list of strings into a NELF string.
    fn to_nelf(self) -> Vec<u8>;
}

impl<T: IntoIterator<Item = V>, V: ToCell> ToNelfSealed for T {}

impl<T: IntoIterator<Item = V>, V: ToCell> ToNelf for T {
    fn to_nelf(self) -> Vec<u8> {
        let mut result = Vec::new();

        for string in self.into_iter() {
            result.append(&mut string.to_cell());
        }

        result
    }
}

mod private {
    pub trait ToCellSealed {}
    pub trait ToNelfSealed {}
}

#[cfg(test)]
mod tests {
    use crate::ToCell;

    use super::NelfIter;

    #[test]
    fn nelf_iter_1() {
        assert_eq!(NelfIter::from_string(b"C|A|C").collect::<Vec<_>>(), [b"A"]);
        assert_eq!(NelfIter::from_string(b"|A|").collect::<Vec<_>>(), [b"A"]);
        assert_eq!(
            NelfIter::from_string(b"C||A||C||B||C").collect::<Vec<_>>(),
            [b"A", b"B"]
        );
    }

    #[test]
    fn nelf_iter_2() {
        assert_eq!(
            NelfIter::from_string(b"C/A\\C").collect::<Vec<_>>(),
            [b"A"]
        );
        assert_eq!(
            NelfIter::from_string(b"C\\A/C").collect::<Vec<_>>(),
            [b"A"]
        );
        assert_eq!(NelfIter::from_string(b"/A\\").collect::<Vec<_>>(), [b"A"]);
        assert_eq!(NelfIter::from_string(b"\\A/").collect::<Vec<_>>(), [b"A"]);
        assert_eq!(
            NelfIter::from_string(b"C//A\\\\C\\\\B//C").collect::<Vec<_>>(),
            [b"A", b"B"]
        );
    }

    #[test]
    fn nelf_iter_3() {
        assert_eq!(NelfIter::from_string(b"123").next(), None);
        assert_eq!(
            NelfIter::from_string(b"|ABC").collect::<Vec<_>>(),
            [b"ABC"]
        );
        assert_eq!(
            NelfIter::from_string(b"/ABC").collect::<Vec<_>>(),
            [b"ABC"]
        );
        assert_eq!(
            NelfIter::from_string(b"\\ABC").collect::<Vec<_>>(),
            [b"ABC"]
        );
    }

    #[test]
    fn nelf_iter_4() {
        assert_eq!(
            NelfIter::from_string(b"||A|A||").collect::<Vec<_>>(),
            [b"A|A"]
        );
        assert_eq!(
            NelfIter::from_string(b"|A/\\A|").collect::<Vec<_>>(),
            [b"A/\\A"]
        );
        assert_eq!(NelfIter::from_string(b"/|\\").collect::<Vec<_>>(), [b"|"]);
        assert_eq!(NelfIter::from_string(b"\\|/").collect::<Vec<_>>(), [b"|"]);
    }

    #[test]
    fn nelf_cell_1() {
        assert_eq!(b"|".to_cell(), b"/|\\");
        assert_eq!(b"/".to_cell(), b"|/|");
        assert_eq!(b"\\".to_cell(), b"|\\|");
        assert_eq!(b"||".to_cell(), b"/||\\");
        assert_eq!(b"//".to_cell(), b"|//|");
        assert_eq!(b"\\\\".to_cell(), b"|\\\\|");
    }

    #[test]
    fn nelf_cell_2() {
        assert_eq!(b"/|".to_cell(), b"\\\\/|//");
        assert_eq!(b"\\|".to_cell(), b"//\\|\\\\");
        assert_eq!(b"|/\\|".to_cell(), b"//|/\\|\\\\");
        assert_eq!(b"/|/".to_cell(), b"||/|/||");
    }
}
