use std::{io, path::Path};

use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_semantic::{IsGlobalReference, Scoping};
use oxc_span::Span;
use oxc_str::static_ident;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

use crate::{config::GlobalValue, context::LintContext};

mod comment;
mod config;
mod control_flow;
mod declarator;
mod express;
mod jest;
mod jsdoc;
mod nextjs;
mod node;
mod number;
mod promise;
mod react;
mod react_compiler;
mod react_perf;
mod regex;
mod schemars;
mod static_value;
mod this_expression;
mod typescript;
mod unicorn;
mod url;
mod vitest;
mod vue;
pub mod vue_casing;

pub use self::{
    comment::*, config::*, control_flow::*, declarator::*, express::*, jest::*, jsdoc::*,
    nextjs::*, node::*, number::*, promise::*, react::*, react_compiler::*, react_perf::*,
    regex::*, schemars::*, static_value::*, this_expression::*, typescript::*, unicorn::*, url::*,
    vitest::*, vue::*,
};

/// List of Eslint rules that have TypeScript equivalents.
// When adding a new rule to this list, please ensure oxlint-migrate is also updated.
// See https://github.com/oxc-project/oxlint-migrate/blob/659b461eaf5b2f8a7283822ae84a5e619c86fca3/src/constants.ts#L24
// NOTE: Ensure this list is always alphabetized, otherwise the binary_search won't work.
const TYPESCRIPT_COMPATIBLE_ESLINT_RULES: [&str; 18] = [
    "class-methods-use-this",
    "default-param-last",
    "init-declarations",
    "max-params",
    "no-array-constructor",
    "no-dupe-class-members",
    "no-empty-function",
    "no-invalid-this",
    "no-loop-func",
    "no-loss-of-precision",
    "no-magic-numbers",
    "no-redeclare",
    "no-restricted-imports",
    "no-shadow",
    "no-unused-expressions",
    "no-unused-vars",
    "no-use-before-define",
    "no-useless-constructor",
    // these rules are equivalents, but not supported
    // "block-spacing",
    // "brace-style",
    // "comma-dangle",
    // "comma-spacing",
    // "func-call-spacing",
    // "indent",
    // "key-spacing",
    // "keyword-spacing",
    // "lines-around-comment",
    // "lines-between-class-members",
    // "no-extra-parens",
    // "no-extra-semi",
    // "object-curly-spacing",
    // "padding-line-between-statements",
    // "quotes",
    // "semi",
    // "space-before-blocks",
    // "space-before-function-paren",
    // "space-infix-ops"
];

/// Check if the Eslint rule is adapted to Typescript.
/// Many Typescript rule are essentially ports of Eslint plugin rules with minor modifications.
/// For these rules, we use the corresponding eslint rules with some adjustments for compatibility.
pub fn is_eslint_rule_adapted_to_typescript(rule_name: &str) -> bool {
    TYPESCRIPT_COMPATIBLE_ESLINT_RULES.binary_search(&rule_name).is_ok()
}

/// Pads replacement text with spaces when needed to preserve token boundaries
/// with neighboring source characters.
pub fn pad_fix_with_token_boundary(source_text: &str, span: Span, replacement: &mut String) {
    if replacement.is_empty() {
        return;
    }

    let source_bytes = source_text.as_bytes();
    let replacement_bytes = replacement.as_bytes();
    let needs_pad_start = span.start > 0
        && is_identifier_part(source_bytes[span.start as usize - 1] as char)
        && is_identifier_part(replacement.chars().next().unwrap());
    let needs_pad_end = (span.end as usize) < source_bytes.len()
        && is_identifier_start(source_bytes[span.end as usize] as char)
        && !replacement_bytes.last().unwrap().is_ascii_whitespace();

    if needs_pad_start {
        replacement.insert(0, ' ');
    }
    if needs_pad_end {
        replacement.push(' ');
    }
}

/// Checks whether an identifier references a global enabled by the linter configuration.
pub fn is_enabled_global_reference(ident: &IdentifierReference<'_>, ctx: &LintContext<'_>) -> bool {
    ident.is_global_reference(ctx.scoping())
        && ctx
            .get_global_variable_value(ident.name.as_str())
            .is_some_and(|value| value != GlobalValue::Off)
}

pub fn is_string_raw_member_expression(expr: &Expression, scoping: &Scoping) -> bool {
    if let Some(member) = expr.get_member_expr()
        && member.static_property_name().is_some_and(|name| name == "raw")
        && let Expression::Identifier(ident) = member.object().get_inner_expression()
    {
        ident.is_global_reference_name(static_ident!("String"), scoping)
    } else {
        false
    }
}

/// Checks if `haystack` starts with `prefix`, ignoring ASCII case.
pub fn starts_with_ignore_case(haystack: &[u8], prefix: &str) -> bool {
    haystack.get(..prefix.len()).is_some_and(|start| start.eq_ignore_ascii_case(prefix.as_bytes()))
}

/// Reads the content of a path and returns it.
/// This function is faster than native `fs:read_to_string`.
///
/// # Errors
/// When the content of the path is not a valid UTF-8 bytes
pub fn read_to_string(path: &Path) -> io::Result<String> {
    // `simdutf8` is faster than `std::str::from_utf8` which `fs::read_to_string` uses internally
    let bytes = std::fs::read(path)?;
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        // Same error as `fs::read_to_string` produces (`io::Error::INVALID_UTF8`)
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}

#[cfg(test)]
mod test {
    use crate::utils::TYPESCRIPT_COMPATIBLE_ESLINT_RULES;

    #[test]
    fn test_typescript_rules_list_is_alphabetized() {
        assert!(TYPESCRIPT_COMPATIBLE_ESLINT_RULES.is_sorted());
    }
}
