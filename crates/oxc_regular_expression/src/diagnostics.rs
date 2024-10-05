use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

const PREFIX: &str = "Invalid regular expression:";

#[cold]
pub fn duplicated_capturing_group_names(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Duplicated capturing group names")).with_labels(spans)
}

#[cold]
pub fn too_may_capturing_groups(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Too many capturing groups")).with_label(span)
}

#[cold]
pub fn parse_pattern_incomplete(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Could not parse the entire pattern")).with_label(span)
}

#[cold]
pub fn lone_quantifier(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Lone quantifier found, expected with `{kind}`"))
        .with_label(span)
}

#[cold]
pub fn unterminated_pattern(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unterminated {kind}")).with_label(span)
}

#[cold]
pub fn invalid_extended_atom_escape(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid extended atom escape")).with_label(span)
}

#[cold]
pub fn invalid_braced_quantifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid braced quantifier")).with_label(span)
}

#[cold]
pub fn invalid_indexed_reference(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid indexed reference")).with_label(span)
}

#[cold]
pub fn empty_group_specifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Group specifier is empty")).with_label(span)
}

#[cold]
pub fn invalid_named_reference(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid named reference")).with_label(span)
}

#[cold]
pub fn invalid_unicode_property_name_negative_strings(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "{PREFIX} Invalid property name `{name}`(negative + property of strings)"
    ))
    .with_label(span)
}

#[cold]
pub fn invalid_character_class(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid character class with strings unicode property"))
        .with_label(span)
}

#[cold]
pub fn character_class_range_out_of_order(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Character {kind} range out of order")).with_label(span)
}

#[cold]
pub fn character_class_range_invalid_atom(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Character class range with invalid atom"))
        .with_label(span)
}

#[cold]
pub fn invalid_class_atom(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid class atom")).with_label(span)
}

#[cold]
pub fn empty_class_set_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Expected nonempty class set expression"))
        .with_label(span)
}

#[cold]
pub fn class_intersection_unexpected_ampersand(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unexpected `&` inside of class intersection"))
        .with_label(span)
}

#[cold]
pub fn class_set_expression_invalid_character(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unexpected character inside of {kind}")).with_label(span)
}

#[cold]
pub fn character_class_contents_invalid_operands(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "{PREFIX} Invalid class operands inside of character class contents"
    ))
    .with_label(span)
}

#[cold]
pub fn missing_capturing_group_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Missing capturing group name")).with_label(span)
}

#[cold]
pub fn too_large_number_in_braced_quantifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Number is too large in braced quantifier"))
        .with_label(span)
}

#[cold]
pub fn braced_quantifier_out_of_order(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Numbers out of order in braced quantifier"))
        .with_label(span)
}

#[cold]
pub fn too_large_number_digits(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Number is too large in {kind} digits")).with_label(span)
}

#[cold]
pub fn invalid_unicode_property(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode property {kind}")).with_label(span)
}

#[cold]
pub fn invalid_unicode_property_of_strings(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode property `{name}`"))
        .with_help("Enable `UnicodeSetsMode` to use this property")
        .with_label(span)
}

#[cold]
pub fn invalid_unicode_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode escape sequence")).with_label(span)
}

#[cold]
pub fn invalid_surrogate_pair(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid surrogate pair")).with_label(span)
}
