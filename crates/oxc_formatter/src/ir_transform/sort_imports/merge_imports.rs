use super::sortable_imports::SortableImport;
use super::source_line::{SourceLine, SpecifierInfo};
use crate::{
    JsLabels,
    formatter::format_element::{
        FormatElement, LineMode, TextWidth,
        tag::{self, LabelId, Tag},
    },
    options::Semicolons,
};

pub fn build_merged_import_elements<'a>(
    is_type_import: bool,
    specifiers: &[SpecifierInfo<'a>],
    source: &'a str,
    bracket_spacing: bool,
    semicolons: Semicolons,
) -> Vec<FormatElement<'a>> {
    let mut elements = Vec::new();

    elements.push(FormatElement::Tag(Tag::StartLabelled(LabelId::of(JsLabels::ImportDeclaration))));

    elements.push(FormatElement::Token { text: "import" });
    elements.push(FormatElement::Space);

    if is_type_import {
        elements.push(FormatElement::Token { text: "type" });
        elements.push(FormatElement::Space);
    }

    elements.push(FormatElement::Token { text: "{" });

    if specifiers.len() == 1 {
        if bracket_spacing {
            elements.push(FormatElement::Space);
        }
        push_specifier(&mut elements, &specifiers[0]);
        if bracket_spacing {
            elements.push(FormatElement::Space);
        }
    } else {
        let line_mode = if bracket_spacing { LineMode::SoftOrSpace } else { LineMode::Soft };

        elements.push(FormatElement::Tag(Tag::StartGroup(tag::Group::new())));
        elements.push(FormatElement::Tag(Tag::StartIndent));
        elements.push(FormatElement::Line(line_mode));

        for (i, spec) in specifiers.iter().enumerate() {
            if i > 0 {
                elements.push(FormatElement::Token { text: "," });
                elements.push(FormatElement::Line(line_mode));
            }
            push_specifier(&mut elements, spec);
        }

        elements.push(FormatElement::Tag(Tag::EndIndent));
        elements.push(FormatElement::Line(line_mode));
        elements.push(FormatElement::Tag(Tag::EndGroup));
    }

    elements.push(FormatElement::Token { text: "}" });
    elements.push(FormatElement::Space);
    elements.push(FormatElement::Token { text: "from" });
    elements.push(FormatElement::Space);

    elements.push(FormatElement::Text {
        text: source,
        width: TextWidth::from_non_whitespace_str(source),
    });

    if matches!(semicolons, Semicolons::Always) {
        elements.push(FormatElement::Token { text: ";" });
    }

    elements.push(FormatElement::Tag(Tag::EndLabelled));

    elements
}

fn push_specifier<'a>(elements: &mut Vec<FormatElement<'a>>, spec: &SpecifierInfo<'a>) {
    if spec.is_type {
        elements.push(FormatElement::Token { text: "type" });
        elements.push(FormatElement::Space);
    }

    elements.push(FormatElement::Text {
        text: spec.imported,
        width: TextWidth::from_non_whitespace_str(spec.imported),
    });

    if let Some(local) = spec.local {
        elements.push(FormatElement::Space);
        elements.push(FormatElement::Token { text: "as" });
        elements.push(FormatElement::Space);
        elements.push(FormatElement::Text {
            text: local,
            width: TextWidth::from_non_whitespace_str(local),
        });
    }
}

/// Merge adjacent duplicate imports in a sorted import list.
///
/// Groups consecutive imports with the same (normalized_source, is_type_import),
/// where each import is "mergeable" (only named specifiers, no default/namespace/side-effect).
/// Specifiers are deduplicated by (imported, local, is_type).
pub fn merge_adjacent_duplicates<'a>(
    imports: Vec<SortableImport<'a>>,
    bracket_spacing: bool,
    semicolons: Semicolons,
) -> Vec<SortableImport<'a>> {
    if imports.len() < 2 {
        return imports;
    }

    let mut result: Vec<SortableImport<'a>> = Vec::with_capacity(imports.len());
    let mut iter = imports.into_iter();
    let mut current_group: Vec<SortableImport<'a>> = vec![iter.next().unwrap()];

    for import in iter {
        let last = current_group.last().unwrap();
        let can_merge = is_mergeable_import(last)
            && is_mergeable_import(&import)
            && last.normalized_source == import.normalized_source
            && get_is_type_import(last) == get_is_type_import(&import);

        if can_merge {
            current_group.push(import);
        } else {
            flush_group(&mut result, current_group, bracket_spacing, semicolons);
            current_group = vec![import];
        }
    }
    flush_group(&mut result, current_group, bracket_spacing, semicolons);

    result
}

fn flush_group<'a>(
    result: &mut Vec<SortableImport<'a>>,
    group: Vec<SortableImport<'a>>,
    bracket_spacing: bool,
    semicolons: Semicolons,
) {
    if group.len() == 1 {
        result.extend(group);
    } else {
        result.push(merge_group(group, bracket_spacing, semicolons));
    }
}

fn merge_group(
    group: Vec<SortableImport<'_>>,
    bracket_spacing: bool,
    semicolons: Semicolons,
) -> SortableImport<'_> {
    debug_assert!(group.len() >= 2);

    let mut group_iter = group.into_iter();
    let first = group_iter.next().unwrap();

    // Extract metadata from the first import
    let (source, is_type_import, mut all_specifiers) = match first.import_line {
        SourceLine::Import(_, meta) => (meta.source, meta.is_type_import, meta.specifiers),
        _ => unreachable!("merge_group called with non-Import line"),
    };

    let normalized_source = first.normalized_source;
    let group_idx = first.group_idx;
    let leading_lines = first.leading_lines;

    // Add specifiers from remaining imports, deduplicating by (imported, local, is_type)
    for import in group_iter {
        if let SourceLine::Import(_, meta) = import.import_line {
            for spec in meta.specifiers {
                let is_duplicate = all_specifiers.iter().any(|s| {
                    s.imported == spec.imported
                        && s.local == spec.local
                        && s.is_type == spec.is_type
                });
                if !is_duplicate {
                    all_specifiers.push(spec);
                }
            }
        }
    }

    let elements = build_merged_import_elements(
        is_type_import,
        &all_specifiers,
        source,
        bracket_spacing,
        semicolons,
    );

    SortableImport {
        leading_lines,
        import_line: SourceLine::MergedImport(elements),
        group_idx,
        normalized_source,
        is_side_effect: false,
        is_ignored: false,
    }
}

fn is_mergeable_import(import: &SortableImport) -> bool {
    match &import.import_line {
        SourceLine::Import(_, meta) => {
            meta.has_named_specifier
                && !meta.has_default_specifier
                && !meta.has_namespace_specifier
                && !meta.is_side_effect
        }
        _ => false,
    }
}

fn get_is_type_import(import: &SortableImport) -> bool {
    match &import.import_line {
        SourceLine::Import(_, meta) => meta.is_type_import,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::super::source_line::ImportLineMetadata;
    use super::*;

    #[test]
    fn two_specifiers() {
        let specs = vec![
            SpecifierInfo { imported: "a", local: None, is_type: false },
            SpecifierInfo { imported: "b", local: None, is_type: false },
        ];

        let elements = build_merged_import_elements(false, &specs, "'x'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);

        assert_eq!(tokens, vec!["import", "{", "a", ",", "b", "}", "from", "'x'", ";"]);
    }

    #[test]
    fn single_specifier() {
        let specs = vec![SpecifierInfo { imported: "a", local: None, is_type: false }];

        let elements = build_merged_import_elements(false, &specs, "'x'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);
        assert_eq!(tokens, vec!["import", "{", "a", "}", "from", "'x'", ";"]);

        // No Group/Indent tags
        let has_group =
            elements.iter().any(|el| matches!(el, FormatElement::Tag(Tag::StartGroup(_))));
        assert!(!has_group, "single specifier should not use group");
    }

    #[test]
    fn specifier_with_alias() {
        let specs = vec![SpecifierInfo { imported: "join", local: Some("j"), is_type: false }];

        let elements =
            build_merged_import_elements(false, &specs, "'path'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);

        assert_eq!(tokens, vec!["import", "{", "join", "as", "j", "}", "from", "'path'", ";"]);
    }

    #[test]
    fn per_specifier_type() {
        let specs = vec![
            SpecifierInfo { imported: "Foo", local: None, is_type: true },
            SpecifierInfo { imported: "bar", local: None, is_type: false },
        ];

        let elements = build_merged_import_elements(false, &specs, "'x'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);

        assert_eq!(tokens, vec!["import", "{", "type", "Foo", ",", "bar", "}", "from", "'x'", ";"]);
    }

    #[test]
    fn type_import() {
        let specs = vec![SpecifierInfo { imported: "A", local: None, is_type: false }];

        let elements = build_merged_import_elements(true, &specs, "'x'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);

        assert_eq!(tokens, vec!["import", "type", "{", "A", "}", "from", "'x'", ";"]);
    }

    #[test]
    fn no_semicolon() {
        let specs = vec![SpecifierInfo { imported: "a", local: None, is_type: false }];

        let elements =
            build_merged_import_elements(false, &specs, "'x'", true, Semicolons::AsNeeded);
        let tokens = collect_tokens(&elements);

        assert_eq!(tokens, vec!["import", "{", "a", "}", "from", "'x'"]);
    }

    #[test]
    fn no_bracket_spacing() {
        let specs = vec![SpecifierInfo { imported: "a", local: None, is_type: false }];

        let elements =
            build_merged_import_elements(false, &specs, "'x'", false, Semicolons::Always);
        // Should be no Space between `{` and `a` and between `a` and `}`
        let has_space_around_brace = elements.windows(2).any(|w| {
            matches!(
                (&w[0], &w[1]),
                (FormatElement::Token { text: "{" }, FormatElement::Space)
                    | (FormatElement::Space, FormatElement::Token { text: "}" })
            )
        });

        assert!(!has_space_around_brace, "no spaces around braces when bracket_spacing=false");
    }

    fn collect_tokens<'a>(elements: &[FormatElement<'a>]) -> Vec<&'a str> {
        elements
            .iter()
            .filter_map(|el| match el {
                FormatElement::Token { text } => Some(*text),
                FormatElement::Text { text, .. } => Some(*text),
                _ => None,
            })
            .collect()
    }

    fn make_import<'a>(
        source: &'a str,
        is_type_import: bool,
        specifiers: Vec<SpecifierInfo<'a>>,
    ) -> SortableImport<'a> {
        SortableImport {
            leading_lines: vec![],
            import_line: SourceLine::Import(
                0..0,
                ImportLineMetadata {
                    source,
                    is_side_effect: false,
                    is_type_import,
                    has_default_specifier: false,
                    has_namespace_specifier: false,
                    has_named_specifier: true,
                    specifiers,
                },
            ),
            group_idx: 0,
            normalized_source: Cow::Borrowed(source),
            is_side_effect: false,
            is_ignored: false,
        }
    }

    fn spec(name: &str) -> SpecifierInfo<'_> {
        SpecifierInfo { imported: name, local: None, is_type: false }
    }

    fn assert_merged_specifiers(import: &SortableImport, expected: &[&str]) {
        match &import.import_line {
            SourceLine::MergedImport(elements) => {
                let texts: Vec<&str> = elements
                    .iter()
                    .filter_map(|el| match el {
                        FormatElement::Text { text, .. } => Some(*text),
                        _ => None,
                    })
                    .filter(|t| !t.starts_with('\'') && !t.starts_with('"'))
                    .collect();
                assert_eq!(texts, expected);
            }
            _ => panic!("expected MergedImport, got {import:?}"),
        }
    }

    #[test]
    fn merge_two_different_specifiers() {
        let imports = vec![
            make_import("'x'", false, vec![spec("a")]),
            make_import("'x'", false, vec![spec("b")]),
        ];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 1);
        assert_merged_specifiers(&result[0], &["a", "b"]);
    }

    #[test]
    fn merge_deduplicates_specifiers() {
        let imports = vec![
            make_import("'x'", false, vec![spec("a")]),
            make_import("'x'", false, vec![spec("a"), spec("b")]),
        ];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 1);
        assert_merged_specifiers(&result[0], &["a", "b"]);
    }

    #[test]
    fn no_merge_different_source() {
        let imports = vec![
            make_import("'x'", false, vec![spec("a")]),
            make_import("'y'", false, vec![spec("b")]),
        ];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn no_merge_default_import() {
        let mut import = make_import("'x'", false, vec![spec("a")]);
        if let SourceLine::Import(_, ref mut meta) = import.import_line {
            meta.has_default_specifier = true;
        }
        let imports = vec![import, make_import("'x'", false, vec![spec("b")])];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn no_merge_namespace_import() {
        let mut import = make_import("'x'", false, vec![]);
        if let SourceLine::Import(_, ref mut meta) = import.import_line {
            meta.has_namespace_specifier = true;
            meta.has_named_specifier = false;
        }
        let imports = vec![import, make_import("'x'", false, vec![spec("b")])];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn no_merge_type_vs_value() {
        let imports = vec![
            make_import("'x'", true, vec![spec("A")]),
            make_import("'x'", false, vec![spec("b")]),
        ];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn merge_three_imports_same_source() {
        let imports = vec![
            make_import("'x'", false, vec![spec("a")]),
            make_import("'x'", false, vec![spec("b")]),
            make_import("'x'", false, vec![spec("c")]),
        ];
        let result = merge_adjacent_duplicates(imports, true, Semicolons::Always);
        assert_eq!(result.len(), 1);
        assert_merged_specifiers(&result[0], &["a", "b", "c"]);
    }
}
