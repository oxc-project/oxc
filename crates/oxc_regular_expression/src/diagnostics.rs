use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

const PREFIX: &str = "Invalid regular expression:";

#[cold]
pub fn invalid_input(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid input string literal")).with_label(span)
}

// ---

#[cold]
pub fn unknown_flag(span: Span, flag: &str, valid_flags: &[&str]) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unknown flag: `{flag}` found"))
        .with_label(span)
        .with_help(format!("Valid flags are: {}", valid_flags.join(", ")))
}

#[cold]
pub fn duplicated_flags(span: Span, flag: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Duplicated flag: `{flag}` found"))
        .with_label(span)
        .with_help("Remove the duplicate flag")
}

#[cold]
pub fn invalid_unicode_flags(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode flags combination `u` and `v`"))
        .with_label(span)
        .with_help("The `v` flag is a superset of `u`. Use only either of them")
}

// ---

#[cold]
pub fn duplicated_capturing_group_names(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Duplicated capturing group names"))
        .with_labels(spans)
        .with_help("Duplicate names must be in different alternatives")
}

#[cold]
pub fn too_may_capturing_groups(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Too many capturing groups"))
        .with_label(span)
        .with_help("Use non-capturing groups `(?:...)` to reduce the count")
}

#[cold]
pub fn parse_pattern_incomplete(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Could not parse the entire pattern"))
        .with_label(span)
        .with_help("Check for unbalanced brackets or invalid escape sequences")
}

#[cold]
pub fn lone_quantifier(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Lone quantifier found, expected with `{kind}`"))
        .with_label(span)
        .with_help("Quantifiers must follow an expression. To match literally, escape it with `\\`")
}

#[cold]
pub fn unterminated_pattern(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unterminated {kind}"))
        .with_label(span)
        .with_help("Add the missing closing delimiter")
}

#[cold]
pub fn invalid_extended_atom_escape(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid extended atom escape"))
        .with_label(span)
        .with_help("Use a valid escape sequence or remove the backslash")
}

#[cold]
pub fn invalid_braced_quantifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid braced quantifier"))
        .with_label(span)
        .with_help("Quantifiers must follow an expression. To match literally, escape it with `\\`")
}

#[cold]
pub fn invalid_indexed_reference(span: Span, max: usize) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid indexed reference")).with_label(span).with_help(
        match max {
            0 => "There are no capturing groups defined in this pattern".into(),
            1 => "This pattern only has 1 capturing group".into(),
            _ => format!("This pattern only has {max} capturing groups"),
        },
    )
}

#[cold]
pub fn empty_group_specifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Group specifier is empty"))
        .with_label(span)
        .with_help("Provide a name for the capturing group: `(?<name>...)`")
}

#[cold]
pub fn invalid_named_reference(span: Span, names: &[&str]) -> OxcDiagnostic {
    let diagnostic =
        OxcDiagnostic::error(format!("{PREFIX} Invalid named reference")).with_label(span);
    if names.is_empty() {
        diagnostic.with_help("There are no named capturing groups defined in this pattern")
    } else {
        diagnostic.with_help(format!("Valid group names are: {}", names.join(", ")))
    }
}

#[cold]
pub fn invalid_unicode_property_name_negative_strings(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "{PREFIX} Property name `{name}` is not allowed here"
    ))
    .with_label(span)
    .with_help("Negated character classes cannot contain properties that match string related unicode properties")
}

#[cold]
pub fn invalid_character_class(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid character class with strings unicode property"))
        .with_label(span)
        .with_help("Properties that match string related unicode properties require the `v` flag")
}

#[cold]
pub fn character_class_range_out_of_order(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Character {kind} range out of order"))
        .with_label(span)
        .with_help("In a character range `[a-z]`, the first character must have a smaller code point than the second")
}

#[cold]
pub fn character_class_range_dash_not_subtraction(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Character class range out of order"))
        .with_label(span)
        .with_help("If you meant to use set subtraction `--`, enable the `v` flag")
}

#[cold]
pub fn character_class_range_invalid_atom(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Character class range with invalid atom"))
        .with_label(span)
        .with_help("Character class ranges must use single characters, not character classes or escape sequences like `\\d`")
}

#[cold]
pub fn invalid_class_atom(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid class atom"))
        .with_label(span)
        .with_help("This character or escape sequence is not valid inside a character class")
}

#[cold]
pub fn empty_class_set_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Expected nonempty class set expression"))
        .with_label(span)
        .with_help("Add characters or properties to the class set expression")
}

#[cold]
pub fn class_intersection_unexpected_ampersand(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unexpected `&` inside of class intersection"))
        .with_label(span)
        .with_help("A third `&` is not allowed after `&&`. Did you mean `[a&&[b]]`?")
}

#[cold]
pub fn class_set_expression_invalid_character(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unexpected character inside of {kind}"))
        .with_label(span)
        .with_help("Escape this character with `\\` or remove it")
}

#[cold]
pub fn character_class_contents_invalid_operands(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "{PREFIX} Invalid class operands inside of character class contents"
    ))
    .with_label(span)
    .with_help("Negated character classes cannot contain string related unicode properties")
}

#[cold]
pub fn too_large_number_in_braced_quantifier(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Number is too large in braced quantifier"))
        .with_label(span)
        .with_help("Use a smaller number in the quantifier")
}

#[cold]
pub fn braced_quantifier_out_of_order(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Numbers out of order in braced quantifier"))
        .with_label(span)
        .with_help("In `{n,m}`, the minimum `n` must be less than or equal to the maximum `m`")
}

#[cold]
pub fn too_large_number_digits(span: Span, kind: &str) -> OxcDiagnostic {
    let help = if kind == "hex" {
        "Use a valid code point value (0x0 to 0x10FFFF)"
    } else {
        "The number exceeds the maximum allowed value"
    };
    OxcDiagnostic::error(format!("{PREFIX} Number is too large in {kind} digits"))
        .with_label(span)
        .with_help(help)
}

#[cold]
pub fn invalid_unicode_property(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode property {kind}"))
        .with_label(span)
        .with_help("Check the property name spelling or use a valid Unicode property")
}

#[cold]
pub fn invalid_unicode_property_of_strings(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode property `{name}`"))
        .with_help("Enable `UnicodeSetsMode` to use this property")
        .with_label(span)
}

#[cold]
pub fn invalid_unicode_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid unicode escape sequence"))
        .with_label(span)
        .with_help("Use `\\uXXXX` or `\\u{XXXXX}` format for unicode escapes")
}

#[cold]
pub fn invalid_surrogate_pair(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid surrogate pair")).with_label(span).with_help(
        "A high surrogate (0xD800-0xDBFF) must be followed by a low surrogate (0xDC00-0xDFFF)",
    )
}

#[cold]
pub fn duplicated_modifiers(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Duplicated modifier"))
        .with_label(span)
        .with_help("Each modifier can only appear once")
}

#[cold]
pub fn invalid_modifiers(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Invalid modifiers"))
        .with_label(span)
        .with_help("A modifier cannot be both enabled and disabled")
}

#[cold]
pub fn empty_modifiers(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Empty modifiers"))
        .with_label(span)
        .with_help("Use `(?:...)` for non-capturing groups or add modifiers")
}

#[cold]
pub fn unknown_modifiers(span: Span, valid_modifiers: &[&str]) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{PREFIX} Unknown modifiers"))
        .with_label(span)
        .with_help(format!("Valid modifiers are: {}", valid_modifiers.join(", ")))
}
