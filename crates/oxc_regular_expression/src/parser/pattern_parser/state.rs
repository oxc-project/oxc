use oxc_span::Atom;
use rustc_hash::FxHashSet;

use crate::parser::reader::Reader;

/// NOTE: Currently all of properties are read-only from outside of this module.
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

// ---

/// Returns: Result<(num_of_left_parens, capturing_group_names), duplicated_named_capturing_group_offsets>
fn parse_capturing_groups<'a>(
    reader: &mut Reader<'a>,
) -> Result<(u32, FxHashSet<Atom<'a>>), DuplicatedNamedCapturingGroupOffsets> {
    // Count only normal CapturingGroup(named, unnamed)
    //   (?<name>...), (...)
    // IgnoreGroup, and LookaroundAssertions are ignored
    //   (?:...)
    //   (?=...), (?!...), (?<=...), (?<!...)
    let mut num_of_left_capturing_parens = 0;

    // Track all named groups with their depth and alternative path
    let mut named_groups: Vec<NamedGroupInfo<'a>> = Vec::new();
    let mut group_names: FxHashSet<Atom<'a>> = FxHashSet::default();

    // Track alternatives and depth
    let mut tracker = AlternativeTracker::new();

    let mut in_escape = false;
    let mut in_character_class = false;
    while let Some(cp) = reader.peek() {
        if in_escape {
            in_escape = false;
        } else if cp == '\\' as u32 {
            in_escape = true;
        } else if cp == '[' as u32 {
            in_character_class = true;
        } else if cp == ']' as u32 {
            in_character_class = false;
        } else if !in_character_class && cp == '|' as u32 {
            tracker.mark_alternative();
        } else if !in_character_class && cp == ')' as u32 {
            tracker.exit_group();
        } else if !in_character_class && cp == '(' as u32 {
            reader.advance();
            tracker.enter_group();

            // Check for non-capturing groups and lookarounds
            // Note: these still increase depth but don't count as capturing groups
            if reader.eat2('?', ':')
                || reader.eat2('?', '=')
                || reader.eat2('?', '!')
                || reader.eat3('?', '<', '=')
                || reader.eat3('?', '<', '!')
            {
                // Non-capturing group or lookaround - depth already incremented
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
                    let alternative_path = tracker.get_alternative_path();

                    // Check for duplicates with existing groups
                    for existing in &named_groups {
                        if existing.name == group_name {
                            // Check if they can participate together
                            if !AlternativeTracker::can_participate(
                                &existing.alternative_path,
                                &alternative_path,
                            ) {
                                // True duplicate - return error
                                return Err(vec![existing.span, (span_start, span_end)]);
                            }
                        }
                    }

                    named_groups.push(NamedGroupInfo {
                        name: group_name,
                        span: (span_start, span_end),
                        alternative_path,
                    });
                    group_names.insert(group_name);

                    continue;
                }
            }

            // Unnamed
            continue;
        }

        reader.advance();
    }

    Ok((num_of_left_capturing_parens, group_names))
}

/// Tracks which alternatives at each depth level have been seen.
/// Used to determine if duplicate named groups are in different alternatives.
#[derive(Debug)]
struct AlternativeTracker {
    /// Current nesting depth
    depth: u32,
    /// Current alternative index at each depth level (stack-based)
    /// Each level represents the alternative index at that nesting depth
    current_alternative: Vec<u32>,
}

impl AlternativeTracker {
    fn new() -> Self {
        Self { depth: 0, current_alternative: vec![0] }
    }

    fn enter_group(&mut self) {
        self.depth += 1;
        while self.current_alternative.len() <= self.depth as usize {
            self.current_alternative.push(0);
        }
    }

    fn exit_group(&mut self) {
        if let Some(alt) = self.current_alternative.get_mut(self.depth as usize) {
            *alt = 0;
        }
        self.depth = self.depth.saturating_sub(1);
    }

    fn mark_alternative(&mut self) {
        if let Some(alt) = self.current_alternative.get_mut(self.depth as usize) {
            *alt += 1;
        }
    }

    fn get_alternative_path(&self) -> Vec<u32> {
        self.current_alternative.iter().take((self.depth + 1) as usize).copied().collect()
    }

    fn can_participate(alt1: &[u32], alt2: &[u32]) -> bool {
        let min_len = alt1.len().min(alt2.len());
        // Check as prefixes, if they differ at any level,
        // it means they are in different alternatives, so they can participate together.
        for i in 0..min_len {
            if alt1[i] != alt2[i] {
                return true;
            }
        }
        false
    }
}

/// Tracks information about a named capturing group
#[derive(Debug, Clone)]
struct NamedGroupInfo<'a> {
    name: Atom<'a>,
    span: (u32, u32),
    alternative_path: Vec<u32>,
}

// ---

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
            ("(?<n>.)(?<m>.)|(?<n>..)|(?<m>.)", (4, 2)),
            ("(?<n>.)(?<m>.)|(?:..)|(?<m>.)", (3, 2)),
            // Test exit_group reset behavior: consecutive groups at same depth
            ("((?<a>x))((?<b>y))|(?<c>z)", (5, 3)), // 2 outer groups + 2 inner named + 1 named = 5 total
            ("((?<a>x))|((?<a>y))", (4, 1)), // 2 outer + 2 inner named = 4 total, 1 unique name
            // Nested groups with alternatives
            ("((?<a>x)|((?<a>y)))", (4, 1)), // 1 outer + 1 named + 1 inner + 1 named = 4 total
            ("(((?<a>x))|((?<b>y)))|(((?<a>z))|((?<b>w)))", (10, 2)), // Complex nesting
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
        for source_text in [
            "(?<n>.)(?<n>..)",
            "(?<n>.(?<n>..))",
            "|(?<n>.(?<n>..))",
            "(?<m>.)|(?<n>.(?<n>..))",
            // Test consecutive groups with same name in same alternative (should be error)
            "((?<a>x))((?<a>y))((?<a>z))",
            "(?<n>a)((?<n>b))",
        ] {
            let mut reader = Reader::initialize(source_text, true, false).unwrap();

            assert!(parse_capturing_groups(&mut reader).is_err(), "{source_text}");
        }
    }
}
