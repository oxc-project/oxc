use oxc_allocator::Allocator;
use oxc_str::Str;
use rustc_hash::FxHashSet;

use crate::parser::{pattern_parser::group_name::capturing_group_name, reader::Reader};

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
    pub capturing_group_names: FxHashSet<Str<'a>>,
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
        allocator: &'a Allocator,
    ) -> Result<(), DuplicatedNamedCapturingGroupOffsets> {
        // PERF: Fast path, no `(` in the decoded units.
        // Provably no capturing group (named or unnamed) and no duplicate name.
        // `Self::new`'s defaults apply except `named_capture_groups`,
        // which for a group-free pattern depends solely on the `u`/`v` flag.
        //
        // `Reader::initialize`'s decode does NOT collapse regex-level escapes
        // (only string/template literal escapes), so `\(`, `[(]`, `(?:…)` etc.
        // still leave a literal `(` unit and fall into the slow path below.
        // Conservative over-approximation: only provably group-free patterns skip the pre-parse.
        if !reader.contains('(') {
            self.named_capture_groups = self.unicode_mode || self.unicode_sets_mode;
            return Ok(());
        }

        let checkpoint = reader.checkpoint();
        let (num_of_left_capturing_parens, capturing_group_names) =
            parse_capturing_groups(reader, self.unicode_mode, self.unicode_sets_mode, allocator)?;
        reader.rewind(checkpoint);

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
    unicode_mode: bool,
    unicode_sets_mode: bool,
    allocator: &'a Allocator,
) -> Result<(u32, FxHashSet<Str<'a>>), DuplicatedNamedCapturingGroupOffsets> {
    // Count only normal CapturingGroup(named, unnamed)
    //   (?<name>...), (...)
    // IgnoreGroup, and LookaroundAssertions are ignored
    //   (?:...)
    //   (?=...), (?!...), (?<=...), (?<!...)
    let mut num_of_left_capturing_parens = 0;

    // Track all named groups with their depth and alternative path
    let mut named_groups: Vec<NamedGroupInfo<'a>> = Vec::new();
    let mut group_names: FxHashSet<Str<'a>> = FxHashSet::default();

    // Track alternatives and depth
    let mut tracker = AlternativeTracker::new();

    let mut in_escape = false;
    let mut character_class_depth = 0_u32;
    while let Some(cp) = reader.peek() {
        if in_escape {
            in_escape = false;
        } else if cp == '\\' as u32 {
            in_escape = true;
        } else if cp == '[' as u32 {
            // Unicode sets allow nested classes; their string alternatives are not group alternatives.
            if unicode_sets_mode || character_class_depth == 0 {
                character_class_depth += 1;
            }
        } else if cp == ']' as u32 {
            character_class_depth = character_class_depth.saturating_sub(1);
        } else if character_class_depth == 0 && cp == '|' as u32 {
            tracker.mark_alternative();
        } else if character_class_depth == 0 && cp == ')' as u32 {
            tracker.exit_group();
        } else if character_class_depth == 0 && cp == '(' as u32 {
            reader.advance();
            tracker.enter_group();

            // Check for non-capturing groups and lookarounds
            // Note: these still increase depth but don't count as capturing groups
            if reader.eat2('?', ':')
                || reader.eat2('?', '=')
                || reader.eat2('?', '!')
                || reader.eat3('?', '<', '=')
                || reader.eat3('?', '<', '!')
                || reader.eat2('?', 'i')
                || reader.eat2('?', 'm')
                || reader.eat2('?', 's')
                || reader.eat2('?', '-')
            {
                // Non-capturing group, modifier group, or lookaround.
                // Full parsing validates the rest of the modifier prefix.
                continue;
            }

            // Count named or unnamed capturing groups
            num_of_left_capturing_parens += 1;

            // Collect capturing group names
            if reader.eat2('?', '<') {
                let span_start = reader.offset();
                let name_start = reader.checkpoint();
                while let Some(ch) = reader.peek() {
                    if ch == '>' as u32 {
                        break;
                    }
                    reader.advance();
                }
                let span_end = reader.offset();

                if reader.peek() == Some('>' as u32) {
                    let group_name = capturing_group_name(
                        reader.str(span_start, span_end).as_str(),
                        reader.code_points_since(name_start),
                        unicode_mode,
                        allocator,
                    );
                    reader.advance();
                    let Some(group_name) = group_name else {
                        continue;
                    };
                    let alternative_path = tracker.get_alternative_path();

                    // Check for duplicates with existing groups
                    for existing in &named_groups {
                        if existing.name == group_name {
                            // Check if they can participate together
                            if AlternativeTracker::can_participate(
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

/// A position in each enclosing disjunction: (disjunction ID, alternative index).
type AlternativePath = Vec<(usize, usize)>;

/// Tracks alternatives within distinct disjunctions.
/// Different alternatives exclude each other only if they belong to the same disjunction.
#[derive(Debug)]
struct AlternativeTracker {
    next_disjunction_id: usize,
    current_alternative: AlternativePath,
}

impl AlternativeTracker {
    fn new() -> Self {
        Self { next_disjunction_id: 1, current_alternative: vec![(0, 0)] }
    }

    fn enter_group(&mut self) {
        self.current_alternative.push((self.next_disjunction_id, 0));
        self.next_disjunction_id += 1;
    }

    fn exit_group(&mut self) {
        if self.current_alternative.len() > 1 {
            self.current_alternative.pop();
        }
    }

    fn mark_alternative(&mut self) {
        self.current_alternative.last_mut().unwrap().1 += 1;
    }

    fn get_alternative_path(&self) -> AlternativePath {
        // A GroupSpecifier is outside its own group's Disjunction, so only
        // enclosing groups can put it in a mutually exclusive alternative.
        self.current_alternative[..self.current_alternative.len() - 1].to_vec()
    }

    fn can_participate(alt1: &[(usize, usize)], alt2: &[(usize, usize)]) -> bool {
        for ((disjunction1, alternative1), (disjunction2, alternative2)) in alt1.iter().zip(alt2) {
            if disjunction1 != disjunction2 {
                // The paths diverged into separate groups, not alternative branches.
                break;
            }
            if alternative1 != alternative2 {
                return false;
            }
        }
        true
    }
}

/// Tracks information about a named capturing group
#[derive(Debug, Clone)]
struct NamedGroupInfo<'a> {
    name: Str<'a>,
    span: (u32, u32),
    alternative_path: AlternativePath,
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

            let allocator = Allocator::default();
            let (num_of_left_capturing_parens, capturing_group_names) =
                parse_capturing_groups(&mut reader, true, false, &allocator).unwrap();

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

            let allocator = Allocator::default();
            assert!(
                parse_capturing_groups(&mut reader, true, false, &allocator).is_err(),
                "{source_text}"
            );
        }
    }
}
