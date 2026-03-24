//! WTF-8 string types for oxc.
//!
//! [WTF-8](https://simonsapin.github.io/wtf-8/) is a superset of UTF-8 that can represent
//! lone surrogates. It is the natural string encoding for JavaScript, whose strings may contain
//! lone surrogate code units (U+D800..=U+DFFF).
//!
//! # Types
//!
//! | Type | Description | Analogue |
//! |------|-------------|---------|
//! | [`Wtf8`] | Borrowed WTF-8 string slice | `str` |
//! | [`Wtf8Buf`] | Owned growable WTF-8 string | `String` |
//! | [`Wtf8Atom<'a>`][`Wtf8Atom`] | Arena-allocated WTF-8 atom | [`Atom<'a>`][oxc_str::Atom] |
//!
//! # Example
//!
//! ```rust
//! use oxc_allocator::Allocator;
//! use oxc_wtf8::{Wtf8, Wtf8Atom, Wtf8Buf};
//!
//! // Build an owned string with a lone surrogate
//! let mut buf = Wtf8Buf::new();
//! buf.push_str("hello ");
//! buf.push_code_point(0xD800); // lone lead surrogate
//! buf.push_str("!");
//! assert!(buf.contains_lone_surrogates());
//!
//! // Lossily convert to a regular &str
//! let lossy = buf.to_str_lossy();
//! assert_eq!(&*lossy, "hello \u{FFFD}!");
//!
//! // Create an arena-allocated atom from UTF-8
//! let allocator = Allocator::new();
//! let atom = Wtf8Atom::from_str("no surrogates");
//! let regular = atom.try_into_atom().expect("valid UTF-8");
//! assert_eq!(regular.as_str(), "no surrogates");
//! ```

mod wtf8;
mod wtf8_atom;

pub use wtf8::{CodePoint, CodePoints, IllFormedUtf16, Wtf8, Wtf8Buf, validate_wtf8};
pub use wtf8_atom::Wtf8Atom;
