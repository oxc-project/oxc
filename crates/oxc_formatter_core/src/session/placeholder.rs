//! Placeholders: a host's stand-ins for parts of its own in the code it dispatches,
//! found in the child's IR and substituted there (css-in-js's `${}`, html-in-js's, Angular's nested pipes).

use std::fmt;

use crate::FormatElement;

/// A host's stand-in for one of its parts in the code it dispatches: `{prefix}{index}{suffix}`, one per part, in order.
///
/// Its affixes are for the host to pick: one token for the child where it goes, which it prints as written
/// (an ASCII lowercase name: `oxfmt_html_0_1_`).
#[derive(Clone, Copy, Debug)]
pub struct Placeholder<'p> {
    pub prefix: &'p str,
    pub suffix: &'p str,
}

/// A piece of a text a [`Placeholder`] splits: a run between placeholders, or a placeholder's index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceholderPiece<'t> {
    Text(&'t str),
    Index(usize),
}

impl<'p> Placeholder<'p> {
    /// `text` holds the prefix: a placeholder, or what looks like one.
    pub fn is_in(self, text: &str) -> bool {
        text.contains(self.prefix)
    }

    /// Writes the placeholder of `index` (into a string, which cannot fail).
    pub fn write(self, out: &mut impl fmt::Write, index: usize) {
        let _ = write!(out, "{}{index}{}", self.prefix, self.suffix);
    }

    /// The pieces of `text`, in order, no run empty: a prefix not followed by digits and the suffix is a run's.
    pub fn pieces<'t>(
        self,
        text: &'t str,
    ) -> impl Iterator<Item = PlaceholderPiece<'t>> + use<'t, 'p> {
        debug_assert!(!self.prefix.is_empty(), "an empty prefix is in every text");
        let mut rest = text;
        let mut index = None;
        std::iter::from_fn(move || {
            if let Some(index) = index.take() {
                return Some(PlaceholderPiece::Index(index));
            }
            let mut from = 0;
            while let Some(at) = rest[from..].find(self.prefix).map(|i| from + i) {
                let after = &rest[at + self.prefix.len()..];
                let digits = after.bytes().take_while(u8::is_ascii_digit).count();
                if digits > 0
                    && after[digits..].starts_with(self.suffix)
                    && let Ok(found) = after[..digits].parse()
                {
                    let run = &rest[..at];
                    rest = &after[digits + self.suffix.len()..];
                    if run.is_empty() {
                        return Some(PlaceholderPiece::Index(found));
                    }
                    index = Some(found);
                    return Some(PlaceholderPiece::Text(run));
                }
                from = at + self.prefix.len();
            }
            (!rest.is_empty()).then(|| PlaceholderPiece::Text(std::mem::take(&mut rest)))
        })
    }

    /// `ir` holds each placeholder of `0..count` once, in order.
    /// A `BestFitting`'s variants are the same content: each holds the same ones, counted once.
    pub fn are_sequential(self, ir: &[FormatElement<'_>], count: usize) -> bool {
        let mut next = 0;
        self.sequential_from(ir, &mut next) && next == count
    }

    /// [`Self::are_sequential`] from `*next`, which ends one past the last index seen.
    fn sequential_from(self, ir: &[FormatElement<'_>], next: &mut usize) -> bool {
        for element in ir {
            match element {
                FormatElement::Text { text, .. } | FormatElement::Token { text } => {
                    for piece in self.pieces(text.as_str()) {
                        if let PlaceholderPiece::Index(index) = piece {
                            if index != *next {
                                return false;
                            }
                            *next += 1;
                        }
                    }
                }
                FormatElement::BestFitting(best_fitting) => {
                    let start = *next;
                    let mut variants = best_fitting.variants().iter();
                    let Some(first) = variants.next() else { return false };
                    if !self.sequential_from(first, next) {
                        return false;
                    }
                    for variant in variants {
                        let mut variant_next = start;
                        if !self.sequential_from(variant, &mut variant_next)
                            || variant_next != *next
                        {
                            return false;
                        }
                    }
                }
                FormatElement::Interned(interned) if !self.sequential_from(interned, next) => {
                    return false;
                }
                _ => {}
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::{Allocator, ArenaVec};

    use super::{Placeholder, PlaceholderPiece::*};
    use crate::{BestFittingElement, FormatElement, IndentWidth};

    const PH: Placeholder<'static> = Placeholder { prefix: "ph_", suffix: "_" };

    fn best_fitting<'a>(
        allocator: &'a Allocator,
        first: &'a str,
        second: &'a str,
    ) -> FormatElement<'a> {
        let variant = |text| {
            ArenaVec::from_array_in([FormatElement::text(text, IndentWidth::default())], &allocator)
                .into_arena_slice() as &[_]
        };
        let variants = ArenaVec::from_array_in([variant(first), variant(second)], &allocator);
        // SAFETY: Two variants
        FormatElement::BestFitting(unsafe { BestFittingElement::from_vec_unchecked(variants) })
    }

    #[test]
    fn best_fitting_variants_count_once() {
        let allocator = Allocator::default();
        let element = best_fitting(&allocator, "a ph_0_ b ph_1_", "ph_0_ ph_1_");
        assert!(PH.are_sequential(&[element], 2));
    }

    #[test]
    fn best_fitting_variants_hold_the_same() {
        let allocator = Allocator::default();
        let element = best_fitting(&allocator, "ph_0_", "ph_1_");
        assert!(!PH.are_sequential(&[element], 1));
    }

    #[test]
    fn pieces() {
        let pieces = |text| PH.pieces(text).collect::<Vec<_>>();
        assert_eq!(pieces("a ph_0_ b"), [Text("a "), Index(0), Text(" b")]);
        assert_eq!(pieces("ph_0_ph_1_"), [Index(0), Index(1)]);
        assert_eq!(pieces("ph_10__x"), [Index(10), Text("_x")]);
        // No digits, or no suffix: text
        assert_eq!(pieces("ph_ ph_0"), [Text("ph_ ph_0")]);
        assert_eq!(pieces(""), []);
    }
}
