use rustc_hash::FxHashSet;

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
    pub capturing_group_names: FxHashSet<&'a str>,
}

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

    pub fn initialize_with_parsing(&mut self, source_text: &'a str) -> Vec<(usize, usize)> {
        let (
            num_of_left_capturing_parens,
            capturing_group_names,
            duplicated_named_capturing_groups,
        ) = parse_capturing_groups(source_text);

        // In Annex B, this is `false` by default.
        // It is `true`
        // - if `u` or `v` flag is set
        // - or if `GroupName` is found in pattern
        self.named_capture_groups =
            self.unicode_mode || self.unicode_sets_mode || !capturing_group_names.is_empty();

        self.num_of_capturing_groups = num_of_left_capturing_parens;
        self.capturing_group_names = capturing_group_names;

        duplicated_named_capturing_groups
    }
}

/// Returns: (num_of_left_parens, capturing_group_names, duplicated_named_capturing_groups)
fn parse_capturing_groups(source_text: &str) -> (u32, FxHashSet<&str>, Vec<(usize, usize)>) {
    let mut num_of_left_capturing_parens = 0;
    let mut capturing_group_names = FxHashSet::default();
    let mut duplicated_named_capturing_groups = vec![];

    let mut reader = Reader::new(source_text, true);

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
                    let group_name = &source_text[span_start..span_end];
                    // May be duplicated
                    if !capturing_group_names.insert(group_name) {
                        // Report them with `Span`
                        duplicated_named_capturing_groups.push((span_start, span_end));
                    }
                    continue;
                }
            }
        }

        reader.advance();
    }

    (num_of_left_capturing_parens, capturing_group_names, duplicated_named_capturing_groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_capturing_groups() {
        for (source_text, expected) in [
            ("()", (1, 0, false)),
            (r"\1()", (1, 0, false)),
            ("(foo)", (1, 0, false)),
            ("(foo)(bar)", (2, 0, false)),
            ("(foo(bar))", (2, 0, false)),
            ("(foo)[(bar)]", (1, 0, false)),
            (r"(foo)\(bar\)", (1, 0, false)),
            ("(foo)(?<n>bar)", (2, 1, false)),
            ("(foo)(?=...)(?!...)(?<=...)(?<!...)(?:...)", (1, 0, false)),
            ("(foo)(?<n>bar)(?<nn>baz)", (3, 2, false)),
            ("(?<n>.)(?<n>..)", (2, 1, true)),
            ("(?<n>.(?<n>..))", (2, 1, true)),
        ] {
            let (
                num_of_left_capturing_parens,
                capturing_group_names,
                duplicated_named_capturing_groups,
            ) = parse_capturing_groups(source_text);
            let actual = (
                num_of_left_capturing_parens,
                capturing_group_names.len(),
                !duplicated_named_capturing_groups.is_empty(),
            );
            assert_eq!(expected, actual, "{source_text}");
        }
    }
}
