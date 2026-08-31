//! `SortedListRecorder`: the IR permutation primitive behind every member-sorting target.
//!
//! Principle: decide the order on the AST, print members in SOURCE order (comments are
//! consumed through a monotonic cursor, so nodes cannot print out of order), then permute the
//! printed units at the tail of the buffer.
//!
//! The printer brackets each member with `begin_unit` / `end_unit`. Whatever it writes between
//! `end_unit(i)` and `begin_unit(i+1)` is *slot i* (line breaks, `|`/`&` operators, blank lines);
//! slots stay where they are, only units move. Ranges are recorded as buffer positions, never
//! rediscovered by scanning the IR, so multi-line comments or `line_suffix` content inside a unit
//! cannot split it.

use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter_core::format_element::FormatElement;

use crate::{Buffer, formatter::JsFormatter};

#[derive(Debug)]
pub struct SortedListRecorder {
    /// Buffer position where the list starts (before the first unit's leading content).
    start: usize,
    /// `[begin, end)` buffer positions of each unit, in source order.
    units: Vec<(usize, usize)>,
    /// `begin_unit` position of the unit currently being written.
    open_unit_start: Option<usize>,
}

impl SortedListRecorder {
    /// Call right before the first member (and its leading content) is written.
    pub fn start(f: &JsFormatter<'_, '_>) -> Self {
        Self { start: f.elements().len(), units: Vec::new(), open_unit_start: None }
    }

    /// Call right before member `i` is written (before its leading comments).
    pub fn begin_unit(&mut self, f: &JsFormatter<'_, '_>) {
        debug_assert!(self.open_unit_start.is_none(), "begin_unit while a unit is open");
        self.open_unit_start = Some(f.elements().len());
    }

    /// Call right after member `i` is written (after its same-line trailing comments).
    pub fn end_unit(&mut self, f: &JsFormatter<'_, '_>) {
        let begin = self.open_unit_start.take().expect("end_unit without begin_unit");
        let end = f.elements().len();
        debug_assert!(begin <= end);
        debug_assert!(is_tag_balanced(&f.elements()[begin..end]), "unit must be tag-balanced");
        self.units.push((begin, end));
    }

    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    /// Rewrite the tail so units appear in `permutation` order (`permutation[target] = source`).
    /// A no-op for the identity permutation, so already-sorted input is byte-identical.
    pub fn finish(self, f: &mut JsFormatter<'_, '_>, permutation: &[usize]) {
        debug_assert!(self.open_unit_start.is_none(), "finish while a unit is open");
        debug_assert_eq!(permutation.len(), self.units.len());
        if self.units.is_empty() || is_identity(permutation) {
            return;
        }
        let rebuilt = rebuild(
            &f.elements()[self.start..],
            self.start,
            &self.units,
            permutation,
            f.allocator(),
        );
        f.replace_end(self.start, &rebuilt);
    }
}

pub fn is_identity(permutation: &[usize]) -> bool {
    permutation.iter().enumerate().all(|(target, &source)| target == source)
}

/// Every `Tag::Start*` inside `elements` is closed inside it, and nothing closes first.
pub fn is_tag_balanced(elements: &[FormatElement<'_>]) -> bool {
    let mut depth: i64 = 0;
    for element in elements {
        if let FormatElement::Tag(tag) = element {
            if tag.is_start() {
                depth += 1;
            } else if tag.is_end() {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
        }
    }
    depth == 0
}

/// Build the permuted tail: `prefix, unit[p0], slot0, unit[p1], slot1, …, unit[p_{n-1}], suffix`.
///
/// `tail` is `elements[start..]`; `units` are ABSOLUTE buffer positions.
/// Slots are taken from the SOURCE neighbours (`units[k].1 .. units[k+1].0`), so blank lines and
/// separators keep their slot while members move through them.
pub fn rebuild<'a>(
    tail: &[FormatElement<'a>],
    start: usize,
    units: &[(usize, usize)],
    permutation: &[usize],
    allocator: &'a Allocator,
) -> ArenaVec<'a, FormatElement<'a>> {
    debug_assert!(!units.is_empty());
    debug_assert_eq!(units.len(), permutation.len());
    debug_assert!(units.windows(2).all(|w| w[0].1 <= w[1].0), "units must be monotonic");

    let rel = |pos: usize| pos - start;
    let mut out = ArenaVec::with_capacity_in(tail.len(), &allocator);

    out.extend_from_slice(&tail[..rel(units[0].0)]);
    for (target, &source) in permutation.iter().enumerate() {
        let (begin, end) = units[source];
        out.extend_from_slice(&tail[rel(begin)..rel(end)]);
        if let Some(&(next_begin, _)) = units.get(target + 1) {
            let (_, slot_begin) = units[target];
            out.extend_from_slice(&tail[rel(slot_begin)..rel(next_begin)]);
        }
    }
    out.extend_from_slice(&tail[rel(units[units.len() - 1].1)..]);

    debug_assert_eq!(out.len(), tail.len(), "rebuild must not add or drop elements");
    out
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_formatter_core::format_element::{FormatElement, LineMode, tag::Tag};

    use super::*;

    fn tok(text: &'static str) -> FormatElement<'static> {
        FormatElement::Token { text }
    }

    const HARD: FormatElement<'static> = FormatElement::Line(LineMode::Hard);

    #[test]
    fn rebuild_permutes_units_and_keeps_slots_in_place() {
        let allocator = Allocator::default();
        // a <hard> b <empty> c   with units a / b / c, slots: <hard>, <empty>
        let tail = [tok("a"), HARD, tok("b"), FormatElement::Line(LineMode::Empty), tok("c")];
        let units = [(0, 1), (2, 3), (4, 5)];
        let out = rebuild(&tail, 0, &units, &[2, 0, 1], &allocator);
        let expected = [tok("c"), HARD, tok("a"), FormatElement::Line(LineMode::Empty), tok("b")];
        assert_eq!(&out[..], &expected[..]);
    }

    #[test]
    fn rebuild_honors_start_offset_and_multi_element_units() {
        let allocator = Allocator::default();
        // buffer positions 10.. ; unit 0 = [x, y] (10..12), slot = <hard> (12), unit 1 = [z] (13..14)
        let tail = [tok("x"), tok("y"), HARD, tok("z")];
        let units = [(10, 12), (13, 14)];
        let out = rebuild(&tail, 10, &units, &[1, 0], &allocator);
        assert_eq!(&out[..], &[tok("z"), HARD, tok("x"), tok("y")][..]);
    }

    #[test]
    fn rebuild_keeps_prefix_and_suffix_outside_units() {
        let allocator = Allocator::default();
        let tail = [tok("("), tok("a"), tok(","), tok("b"), tok(")")];
        let units = [(1, 2), (3, 4)];
        let out = rebuild(&tail, 0, &units, &[1, 0], &allocator);
        assert_eq!(&out[..], &[tok("("), tok("b"), tok(","), tok("a"), tok(")")][..]);
    }

    #[test]
    fn identity_detection() {
        assert!(is_identity(&[]));
        assert!(is_identity(&[0, 1, 2]));
        assert!(!is_identity(&[1, 0]));
    }

    #[test]
    fn tag_balance_check_accepts_balanced_units() {
        let unit =
            [FormatElement::Tag(Tag::StartIndent), tok("a"), FormatElement::Tag(Tag::EndIndent)];
        assert!(is_tag_balanced(&unit));
        let unbalanced = [FormatElement::Tag(Tag::StartIndent), tok("a")];
        assert!(!is_tag_balanced(&unbalanced));
        let closes_first =
            [FormatElement::Tag(Tag::EndIndent), FormatElement::Tag(Tag::StartIndent)];
        assert!(!is_tag_balanced(&closes_first));
    }
}
