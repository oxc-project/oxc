use oxc_span::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::parser::reader::Reader;

/// Currently all of properties are read only from outside of this module.
/// Even inside of this module, it is not changed after initialized.
#[derive(Debug)]
pub struct State<'a> {
    // Mode flags
    pub unicode_mode: bool,
    pub unicode_sets_mode: bool,
    pub named_capture_groups: bool,
    // Other states
    pub num_of_capturing_groups: u32,
    pub capturing_group_names: FxHashSet<Atom<'a>>,
}

type DuplicatedNamedCapturingGroupOffsets = Vec<(u32, u32)>;

impl<'a> State<'a> {
    pub fn new(unicode_mode: bool, unicode_sets_mode: bool) -> Self {
        Self {
            unicode_mode,
            unicode_sets_mode,
            named_capture_groups: false,
            num_of_capturing_groups: 0,
            capturing_group_names: FxHashSet::default(),
        }
    }

    pub fn initialize_with_parsing(
        &mut self,
        reader: &mut Reader<'a>,
    ) -> Result<(), DuplicatedNamedCapturingGroupOffsets> {
        let (num_of_left_capturing_parens, capturing_group_names) = parse_capturing_groups(reader)?;

        // In Annex B, this is `false` by default.
        // It is `true`
        // - if `u` or `v` flag is set
        // - or if `GroupName` is found in pattern
        self.named_capture_groups =
            self.unicode_mode || self.unicode_sets_mode || !capturing_group_names.is_empty();

        self.num_of_capturing_groups = num_of_left_capturing_parens;
        self.capturing_group_names = capturing_group_names;

        Ok(())
    }
}

/// Returns: Result<
///   (num_of_left_parens, capturing_group_names),
///   duplicated_named_capturing_group_offsets
/// >
fn parse_capturing_groups<'a>(
    reader: &mut Reader<'a>,
) -> Result<(u32, FxHashSet<Atom<'a>>), DuplicatedNamedCapturingGroupOffsets> {
    let mut num_of_left_capturing_parens = 0;
    let mut capturing_group_name_and_spans = FxHashMap::default();

    let mut in_escape = false;
    let mut in_character_class = false;

    // Count only normal CapturingGroup(named, unnamed)
    //   (?<name>...), (...)
    // IgnoreGroup, and LookaroundAssertions are ignored
    //   (?:...)
    //   (?=...), (?!...), (?<=...), (?<!...)
    while let Some(cp) = reader.peek() {
        if in_escape {
            in_escape = false;
        } else if cp == '\\' as u32 {
            in_escape = true;
        } else if cp == '[' as u32 {
            in_character_class = true;
        } else if cp == ']' as u32 {
            in_character_class = false;
        } else if !in_character_class && cp == '(' as u32 {
            reader.advance();

            // Skip IgnoreGroup
            if reader.eat2('?', ':')
            // Skip LookAroundAssertion
                || reader.eat2('?', '=')
                || reader.eat2('?', '!')
                || reader.eat3('?', '<', '=')
                || reader.eat3('?', '<', '!')
            {
                continue;
            }

            // Count named or unnamed capturing groups
            num_of_left_capturing_parens += 1;

            // Collect capturing group names
            if reader.eat2('?', '<') {
                let span_start = reader.offset();
                while let Some(ch) = reader.peek() {
                    if ch == '>' as u32 {
                        break;
                    }
                    reader.advance();
                }
                let span_end = reader.offset();

                if reader.eat('>') {
                    let group_name = reader.atom(span_start, span_end);

                    // May be duplicated
                    if let Some(may_duplicate) = capturing_group_name_and_spans.get(&group_name) {
                        return Err(vec![*may_duplicate, (span_start, span_end)]);
                    }
                    capturing_group_name_and_spans.insert(group_name, (span_start, span_end));

                    continue;
                }
            }

            continue;
        }

        reader.advance();
    }

    Ok((num_of_left_capturing_parens, capturing_group_name_and_spans.keys().cloned().collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_capturing_groups() {
        for (source_text, expected) in [
            ("()", (1, 0)),
            (r"\1()", (1, 0)),
            ("(foo)", (1, 0)),
            ("(foo)(bar)", (2, 0)),
            ("(foo(bar))", (2, 0)),
            ("(foo)[(bar)]", (1, 0)),
            (r"(foo)\(bar\)", (1, 0)),
            ("(foo)(?<n>bar)", (2, 1)),
            ("(foo)(?=...)(?!...)(?<=...)(?<!...)(?:...)", (1, 0)),
            ("(foo)(?<n>bar)(?<nn>baz)", (3, 2)),
        ] {
            let mut reader = Reader::initialize(source_text, true, false).unwrap();

            let (num_of_left_capturing_parens, capturing_group_names) =
                parse_capturing_groups(&mut reader).unwrap();

            let actual = (num_of_left_capturing_parens, capturing_group_names.len());
            assert_eq!(expected, actual, "{source_text}");
        }
    }

    #[test]
    fn duplicated_named_capturing_groups() {
        for source_text in ["(?<n>.)(?<n>..)", "(?<n>.(?<n>..))"] {
            let mut reader = Reader::initialize(source_text, true, false).unwrap();

            assert!(parse_capturing_groups(&mut reader).is_err(), "{source_text}");
        }
    }
}
