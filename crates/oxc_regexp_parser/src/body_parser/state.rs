use rustc_hash::FxHashSet;

use super::reader::Reader;

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
    pub num_of_named_capturing_groups: u32,
    pub found_group_names: FxHashSet<&'a str>,
}

impl<'a> State<'a> {
    pub fn new(unicode_mode: bool, unicode_sets_mode: bool) -> Self {
        Self {
            unicode_mode,
            unicode_sets_mode,
            named_capture_groups: false,
            num_of_capturing_groups: 0,
            num_of_named_capturing_groups: 0,
            found_group_names: FxHashSet::default(),
        }
    }

    pub fn initialize_with_parsing(&mut self, source_text: &'a str) {
        let (num_of_left_parens, num_of_named_capturing_groups, named_capturing_groups) =
            parse_capturing_groups(source_text);

        // In Annex B, this is `false` by default.
        // It is `true`
        // - if `u` or `v` flag is set
        // - or if `GroupName` is found in pattern
        self.named_capture_groups =
            self.unicode_mode || self.unicode_sets_mode || 0 < num_of_named_capturing_groups;

        self.num_of_capturing_groups = num_of_left_parens;
        self.num_of_named_capturing_groups = num_of_named_capturing_groups;
        self.found_group_names = named_capturing_groups;
    }
}

/// Returns: (num_of_left_parens, num_of_named_capturing_groups, named_capturing_groups)
fn parse_capturing_groups(source_text: &str) -> (u32, u32, FxHashSet<&str>) {
    let mut num_of_left_parens = 0;
    let mut num_of_named_capturing_groups = 0;
    let mut named_capturing_groups = FxHashSet::default();

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

            num_of_left_parens += 1;

            // Named capturing group
            if reader.eat2('?', '<') {
                let span_start = reader.span_position();
                while let Some(ch) = reader.peek() {
                    if ch == '>' as u32 {
                        break;
                    }
                    reader.advance();
                }
                let span_end = reader.span_position();

                if reader.eat('>') {
                    let group_name = &source_text[span_start..span_end];
                    // May be duplicated, but it's OK
                    named_capturing_groups.insert(group_name);
                    num_of_named_capturing_groups += 1;
                    continue;
                }
            }
        }

        reader.advance();
    }

    (num_of_left_parens, num_of_named_capturing_groups, named_capturing_groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_capturing_groups() {
        for (source_text, expected_num_of_left_parens, expected_num_of_named_capturing_groups) in [
            ("()", 1, 0),
            (r"\1()", 1, 0),
            ("(foo)", 1, 0),
            ("(foo)(bar)", 2, 0),
            ("(foo(bar))", 2, 0),
            ("(foo)[(bar)]", 1, 0),
            (r"(foo)\(bar\)", 1, 0),
            ("(foo)(?<n>bar)", 2, 1),
            ("(foo)(?=...)(?!...)(?<=...)(?<!...)(?:...)", 1, 0),
            ("(foo)(?<n>bar)(?<nn>baz)", 3, 2),
            ("(?<n>.)(?<n>..)", 2, 2),
        ] {
            let (num_of_left_parens, num_of_named_capturing_groups, _) =
                parse_capturing_groups(source_text);
            assert_eq!(expected_num_of_left_parens, num_of_left_parens);
            assert_eq!(expected_num_of_named_capturing_groups, num_of_named_capturing_groups);
        }
    }
}
