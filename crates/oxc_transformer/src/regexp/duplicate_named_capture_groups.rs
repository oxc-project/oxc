use std::fmt::Write;

use oxc_regular_expression::{
    ast::{Pattern, Term},
    normalize_group_name,
    visit::{RegExpAstKind, Visit, walk},
};
use oxc_span::Span;
use rustc_hash::FxHashMap;

pub(super) struct NamedCaptureGroup {
    pub name: String,
    pub indices: Vec<u32>,
}

pub(super) struct RewriteResult {
    pub pattern: String,
    pub groups: Vec<NamedCaptureGroup>,
}

struct NamedReference {
    span: Span,
    name: String,
    quantified: bool,
}

struct Collector {
    capture_index: u32,
    groups: Vec<NamedCaptureGroup>,
    group_indices: FxHashMap<String, usize>,
    group_specifier_spans: Vec<Span>,
    references: Vec<NamedReference>,
    has_duplicate_names: bool,
    in_quantifier: bool,
}

impl Collector {
    fn new() -> Self {
        Self {
            capture_index: 0,
            groups: Vec::new(),
            group_indices: FxHashMap::default(),
            group_specifier_spans: Vec::new(),
            references: Vec::new(),
            has_duplicate_names: false,
            in_quantifier: false,
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn enter_node(&mut self, kind: RegExpAstKind<'a>) {
        match kind {
            RegExpAstKind::CapturingGroup(group) => {
                self.capture_index += 1;
                let Some(name) = &group.name else {
                    return;
                };

                let normalized_name = normalize_group_name(name.as_str());
                if let Some(&group_index) = self.group_indices.get(normalized_name.as_ref()) {
                    self.groups[group_index].indices.push(self.capture_index);
                    self.has_duplicate_names = true;
                } else {
                    let name = normalized_name.into_owned();
                    self.group_indices.insert(name.clone(), self.groups.len());
                    self.groups.push(NamedCaptureGroup { name, indices: vec![self.capture_index] });
                }

                // Preserve the opening `(` and remove `?<name>`.
                self.group_specifier_spans
                    .push(Span::new(group.span.start + 1, group.body.span.start));
            }
            RegExpAstKind::NamedReference(reference) => {
                self.references.push(NamedReference {
                    span: reference.span,
                    name: normalize_group_name(reference.name.as_str()).into_owned(),
                    quantified: self.in_quantifier,
                });
            }
            _ => {}
        }
    }

    fn visit_quantifier(&mut self, quantifier: &oxc_regular_expression::ast::Quantifier<'a>) {
        let previous = self.in_quantifier;
        self.in_quantifier = matches!(quantifier.body, Term::NamedReference(_));
        walk::walk_quantifier(self, quantifier);
        self.in_quantifier = previous;
    }
}

struct Edit {
    start: usize,
    end: usize,
    replacement: String,
}

/// Rewrite duplicate named capture groups to numbered groups and references.
///
/// Returns `None` when the pattern does not contain duplicate names. The caller can then avoid
/// changing RegExp literals that use only ordinary named groups.
pub(super) fn rewrite_duplicate_named_capture_groups(
    pattern_text: &str,
    pattern: &Pattern<'_>,
    pattern_offset: u32,
) -> Option<RewriteResult> {
    let mut collector = Collector::new();
    collector.visit_pattern(pattern);
    if !collector.has_duplicate_names {
        return None;
    }

    let mut edits =
        Vec::with_capacity(collector.group_specifier_spans.len() + collector.references.len());

    for span in collector.group_specifier_spans {
        edits.push(Edit {
            start: (span.start - pattern_offset) as usize,
            end: (span.end - pattern_offset) as usize,
            replacement: String::new(),
        });
    }

    for reference in collector.references {
        let group_index = collector.group_indices[&reference.name];
        let indices = &collector.groups[group_index].indices;
        let mut replacement = String::new();
        for index in indices {
            write!(replacement, r"\{index}").unwrap();
        }

        let start = (reference.span.start - pattern_offset) as usize;
        let end = (reference.span.end - pattern_offset) as usize;
        let followed_by_digit = pattern_text.as_bytes().get(end).is_some_and(u8::is_ascii_digit);
        if followed_by_digit || (reference.quantified && indices.len() > 1) {
            replacement = format!("(?:{replacement})");
        }

        edits.push(Edit { start, end, replacement });
    }

    edits.sort_unstable_by_key(|edit| edit.start);

    let mut rewritten = String::with_capacity(pattern_text.len());
    let mut cursor = 0;
    for edit in edits {
        debug_assert!(cursor <= edit.start);
        debug_assert!(edit.start <= edit.end);
        debug_assert!(edit.end <= pattern_text.len());
        rewritten.push_str(&pattern_text[cursor..edit.start]);
        rewritten.push_str(&edit.replacement);
        cursor = edit.end;
    }
    rewritten.push_str(&pattern_text[cursor..]);

    Some(RewriteResult { pattern: rewritten, groups: collector.groups })
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_regular_expression::{LiteralParser, Options};

    use super::*;

    type RewriteOutput = (String, Vec<(String, Vec<u32>)>);

    fn rewrite(source: &str) -> Option<RewriteOutput> {
        let allocator = Allocator::default();
        let pattern =
            LiteralParser::new(&allocator, source, None, Options::default()).parse().unwrap();
        rewrite_duplicate_named_capture_groups(source, &pattern, 0).map(|result| {
            let groups =
                result.groups.into_iter().map(|group| (group.name, group.indices)).collect();
            (result.pattern, groups)
        })
    }

    #[test]
    fn rewrites_duplicate_named_capture_groups() {
        assert_eq!(
            rewrite(r"(?<year>\d{4})|(?<year>\d{2})"),
            Some((r"(\d{4})|(\d{2})".to_string(), vec![("year".to_string(), vec![1, 2])],))
        );
        assert_eq!(rewrite(r"(?<year>\d{4})"), None);
    }

    #[test]
    fn rewrites_named_references() {
        assert_eq!(
            rewrite(r"(?:(?<a>x)|(?<a>y))\k<a>"),
            Some((r"(?:(x)|(y))\1\2".to_string(), vec![("a".to_string(), vec![1, 2])],))
        );
        assert_eq!(
            rewrite(r"(?:(?<a>x)|(?<a>y))\k<a>+"),
            Some((r"(?:(x)|(y))(?:\1\2)+".to_string(), vec![("a".to_string(), vec![1, 2])],))
        );
        assert_eq!(
            rewrite(r"(?:(?<a>x)|(?<a>y))\k<a>2"),
            Some((r"(?:(x)|(y))(?:\1\2)2".to_string(), vec![("a".to_string(), vec![1, 2])],))
        );
    }

    #[test]
    fn records_all_named_groups_in_source_order() {
        assert_eq!(
            rewrite(r"(?<y>a)(?<x>a)|(?<x>b)(?<y>b)"),
            Some((
                r"(a)(a)|(b)(b)".to_string(),
                vec![("y".to_string(), vec![1, 4]), ("x".to_string(), vec![2, 3]),],
            ))
        );
        assert_eq!(
            rewrite(r"(.)(?:(?<x>a)|(?<x>b))\k<x>"),
            Some((r"(.)(?:(a)|(b))\2\3".to_string(), vec![("x".to_string(), vec![2, 3])],))
        );
    }

    #[test]
    fn normalizes_escaped_group_names() {
        assert_eq!(
            rewrite(r"(?<a>x)|(?<\u0061>y)\k<\u0061>"),
            Some((r"(x)|(y)\1\2".to_string(), vec![("a".to_string(), vec![1, 2])],))
        );
    }
}
