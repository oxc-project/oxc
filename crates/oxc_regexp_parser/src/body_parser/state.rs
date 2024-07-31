use rustc_hash::FxHashSet;

use super::reader::Reader;

#[derive(Debug)]
pub struct State<'a> {
    pub unicode_mode: bool,
    pub unicode_sets_mode: bool,
    pub named_capture_groups: bool,
    pub num_of_capturing_groups: u32,
    pub found_group_names: FxHashSet<&'a str>,
}

impl<'a> State<'a> {
    pub fn new(unicode_flag: bool, unicode_sets_flag: bool) -> Self {
        let unicode_mode = unicode_flag || unicode_sets_flag;
        let unicode_sets_mode = unicode_sets_flag;
        // In Annex B, this is `false` by default.
        // But it is `true`
        // - if `u` or `v` flag is set
        // - or if `GroupName` is found in pattern
        let named_capture_groups = unicode_flag || unicode_sets_flag;

        Self {
            unicode_mode,
            unicode_sets_mode,
            named_capture_groups,
            num_of_capturing_groups: 0,
            found_group_names: FxHashSet::default(),
        }
    }

    pub fn parse_capturing_groups(&mut self, source_text: &'a str) {
        let (num_of_left_parens, named_capture_groups) = parse_capturing_groups(source_text);

        // Enable `NamedCaptureGroups` if capturing group found
        if !named_capture_groups.is_empty() {
            self.named_capture_groups = true;
        }

        self.num_of_capturing_groups = num_of_left_parens;
        self.found_group_names = named_capture_groups;
    }
}

/// Returns: (num_of_left_parens, named_capture_groups)
fn parse_capturing_groups(source_text: &str) -> (u32, FxHashSet<&str>) {
    let mut num_of_left_parens = 0;
    let mut named_capture_groups = FxHashSet::default();

    let mut reader = Reader::new(source_text, true);

    let mut in_escape = false;
    let mut in_character_class = false;

    // Count only normal capturing group(named, unnamed)
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
                    named_capture_groups.insert(group_name);
                    continue;
                }
            }
        }

        reader.advance();
    }

    (num_of_left_parens, named_capture_groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_capturing_groups() {
        for (source_text, expected_num_of_left_parens, expected_named_capture_groups_found) in [
            ("()", 1, false),
            (r"\1()", 1, false),
            ("(foo)", 1, false),
            ("(foo)(bar)", 2, false),
            ("(foo(bar))", 2, false),
            ("(foo)[(bar)]", 1, false),
            (r"(foo)\(bar\)", 1, false),
            ("(foo)(?<n>bar)", 2, true),
            ("(foo)(?=...)(?!...)(?<=...)(?<!...)(?:...)", 1, false),
            ("(foo)(?<n>bar)(?<nn>baz)", 3, true),
            ("(?<n>.)(?<n>..)", 2, true),
        ] {
            let (num_of_left_parens, named_capture_groups) = parse_capturing_groups(source_text);
            assert_eq!(expected_num_of_left_parens, num_of_left_parens);
            assert_eq!(expected_named_capture_groups_found, !named_capture_groups.is_empty());
        }
    }
}
