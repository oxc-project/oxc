use crate::{
    JsLabels,
    formatter::format_element::{
        FormatElement, LineMode, TextWidth,
        tag::{self, LabelId, Tag},
    },
    options::Semicolons,
};
use super::source_line::SpecifierInfo;

pub fn build_merged_import_elements<'a>(
    is_type_import: bool,
    specifiers: &[SpecifierInfo<'a>],
    source: &'a str,
    bracket_spacing: bool,
    semicolons: Semicolons,
) -> Vec<FormatElement<'a>> {
    let mut elements = Vec::new();
    
    elements.push(FormatElement::Tag(Tag::StartLabelled(
        LabelId::of(JsLabels::ImportDeclaration),
    )));
    
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
        let line_mode = if bracket_spacing {
            LineMode::SoftOrSpace
        } else {
            LineMode::Soft
        };
        
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

#[cfg(test)]
mod tests {
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
        let specs = vec![
            SpecifierInfo { imported: "a", local: None, is_type: false },
        ];
        
        let elements = build_merged_import_elements(false, &specs, "'x'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);
        assert_eq!(tokens, vec!["import", "{", "a", "}", "from", "'x'", ";"]);
        
        // No Group/Indent tags
        let has_group = elements.iter().any(|el| matches!(el, FormatElement::Tag(Tag::StartGroup(_))));
        assert!(!has_group, "single specifier should not use group");
    }
    
    #[test]
    fn specifier_with_alias() {
        let specs = vec![
            SpecifierInfo { imported: "join", local: Some("j"), is_type: false },
        ];
        
        let elements = build_merged_import_elements(false, &specs, "'path'", true, Semicolons::Always);
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
        let specs = vec![
            SpecifierInfo { imported: "A", local: None, is_type: false },
        ];
        
        let elements = build_merged_import_elements(true, &specs, "'x'", true, Semicolons::Always);
        let tokens = collect_tokens(&elements);
        
        assert_eq!(tokens, vec!["import", "type", "{", "A", "}", "from", "'x'", ";"]);
    }
    
    #[test]
    fn no_semicolon() {
        let specs = vec![
            SpecifierInfo { imported: "a", local: None, is_type: false },
        ];
        
        let elements = build_merged_import_elements(false, &specs, "'x'", true, Semicolons::AsNeeded);
        let tokens = collect_tokens(&elements);
        
        assert_eq!(tokens, vec!["import", "{", "a", "}", "from", "'x'"]);
    }
    
    #[test]
    fn no_bracket_spacing() {
        let specs = vec![
            SpecifierInfo { imported: "a", local: None, is_type: false },
        ];

        let elements = build_merged_import_elements(false, &specs, "'x'", false, Semicolons::Always);
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
}