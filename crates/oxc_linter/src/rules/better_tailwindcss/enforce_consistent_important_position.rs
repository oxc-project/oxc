use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn inconsistent_important_position(span: Span, class: &str, fixed: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Incorrect important position. \"{class}\" should be \"{fixed}\""))
        .with_help("Use consistent important modifier position across your codebase")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct EnforceConsistentImportantPosition(Box<EnforceConsistentImportantPositionOptions>);

#[derive(Debug, Clone)]
pub struct EnforceConsistentImportantPositionOptions {
    position: ImportantPosition,
}

impl Default for EnforceConsistentImportantPositionOptions {
    fn default() -> Self {
        Self { position: ImportantPosition::Recommended }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ImportantPosition {
    Legacy,      // !text-red-500
    Recommended, // text-red-500!
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent position for the Tailwind CSS important modifier (`!`).
    ///
    /// ### Why is this bad?
    ///
    /// Mixing different important positions reduces code readability and consistency.
    /// TailwindCSS v4 recommends using the trailing `!` syntax.
    ///
    /// ### Options
    ///
    /// - `position`: `"legacy"` | `"recommended"` (default: `"recommended"`)
    ///   - `"legacy"`: Important modifier at the start (`!text-red-500`)
    ///   - `"recommended"`: Important modifier at the end (`text-red-500!`)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code with `position: "recommended"`:
    /// ```jsx
    /// <div className="!text-red-500 !bg-blue-500" />
    /// ```
    ///
    /// Examples of **correct** code with `position: "recommended"`:
    /// ```jsx
    /// <div className="text-red-500! bg-blue-500!" />
    /// ```
    ///
    /// Examples of **incorrect** code with `position: "legacy"`:
    /// ```jsx
    /// <div className="text-red-500! bg-blue-500!" />
    /// ```
    ///
    /// Examples of **correct** code with `position: "legacy"`:
    /// ```jsx
    /// <div className="!text-red-500 !bg-blue-500" />
    /// ```
    EnforceConsistentImportantPosition,
    better_tailwindcss,
    style,
    fix
);

impl Rule for EnforceConsistentImportantPosition {
    fn from_configuration(value: Value) -> Self {
        let options = value.as_array().and_then(|arr| arr.first());

        let position = options
            .and_then(|v| v.get("position"))
            .and_then(Value::as_str)
            .and_then(|s| match s {
                "legacy" => Some(ImportantPosition::Legacy),
                "recommended" => Some(ImportantPosition::Recommended),
                _ => None,
            })
            .unwrap_or(ImportantPosition::Recommended);

        Self(Box::new(EnforceConsistentImportantPositionOptions { position }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Handle JSX attributes (className, class)
            AstKind::JSXAttribute(attr) => {
                if let Some(name) = attr.name.as_identifier()
                    && matches!(name.name.as_str(), "className" | "class")
                    && let Some(value) = &attr.value
                    && let Some(str_lit) = value.as_string_literal()
                {
                    // For JSX string literals, we need to get the inner span without quotes
                    // The span of the string literal includes the quotes, but we only want to replace the content
                    let content_span = Span::new(str_lit.span.start + 1, str_lit.span.end - 1);
                    self.check_classes(&str_lit.value, content_span, ctx);
                }
            }
            // Handle template literals (for now, just simple string literals in JSX)
            AstKind::TemplateLiteral(template) => {
                // TODO: Handle template literals with expressions
                if template.expressions.is_empty()
                    && template.quasis.len() == 1
                    && let Some(quasi) = template.quasis.first()
                {
                    self.check_classes(&quasi.value.raw, template.span, ctx);
                }
            }
            _ => {}
        }
    }
}

impl EnforceConsistentImportantPosition {
    fn check_classes(&self, class_string: &str, span: Span, ctx: &LintContext) {
        let classes: Vec<&str> = class_string.split_whitespace().collect();

        // Build a fixed version of the entire class string if needed
        let mut has_changes = false;
        let fixed_classes: Vec<String> = classes
            .iter()
            .map(|class| {
                if let Some((fixed_class, _)) = self.check_important_position(class, span) {
                    has_changes = true;
                    fixed_class
                } else {
                    (*class).to_string()
                }
            })
            .collect();

        // If we have fixes to apply, replace the entire class string
        if has_changes {
            let fixed_string = fixed_classes.join(" ");

            // Find the first class that needs fixing for the diagnostic message
            for class in &classes {
                if let Some((fixed_class, _)) = self.check_important_position(class, span) {
                    ctx.diagnostic_with_fix(
                        inconsistent_important_position(span, class, &fixed_class),
                        |fixer| fixer.replace(span, fixed_string.clone()),
                    );
                    break;
                }
            }
        }
    }

    fn check_important_position(&self, class: &str, base_span: Span) -> Option<(String, Span)> {
        // Parse the class to separate variants and base class
        let (variants, base_class) = match class.rsplit_once(':') {
            Some((variants, base)) => (Some(variants), base),
            None => (None, class),
        };

        let has_prefix = base_class.starts_with('!');
        let has_suffix = base_class.ends_with('!');

        // If no important modifier, nothing to check
        if !has_prefix && !has_suffix {
            return None;
        }

        // Check if the position matches our configuration
        let is_correct = match self.0.position {
            ImportantPosition::Legacy => has_prefix && !has_suffix,
            ImportantPosition::Recommended => has_suffix && !has_prefix,
        };

        if is_correct {
            return None;
        }

        // Generate the fixed class
        let fixed_class = self.fix_important_position(class, variants, base_class);

        // For now, return a simplified span (TODO: calculate exact position)
        Some((fixed_class, base_span))
    }

    fn fix_important_position(
        &self,
        _original: &str,
        variants: Option<&str>,
        base_class: &str,
    ) -> String {
        // Remove existing important modifiers
        let clean_base = base_class.trim_start_matches('!').trim_end_matches('!');

        // Add important in the correct position
        let fixed_base = match self.0.position {
            ImportantPosition::Legacy => format!("!{clean_base}"),
            ImportantPosition::Recommended => format!("{clean_base}!"),
        };

        // Reconstruct with variants if present
        if let Some(v) = variants { format!("{v}:{fixed_base}") } else { fixed_base }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    // Test cases for recommended position (default)
    let pass = vec![
        // Correct recommended position
        (r#"<div className="w-4! text-red-500!" />"#, None),
        (r#"<div className="hover:text-red-500!" />"#, None),
        (r#"<div className="sm:hover:bg-blue-500!" />"#, None),
        (r#"<div className="w-[100px]!" />"#, None),
        // No important modifier
        (r#"<div className="w-4 text-red-500" />"#, None),
        (r#"<div className="hover:text-red-500" />"#, None),
        // Correct legacy position with option
        (r#"<div className="!w-4 !text-red-500" />"#, Some(json!([{"position": "legacy"}]))),
        (r#"<div className="hover:!text-red-500" />"#, Some(json!([{"position": "legacy"}]))),
        (r#"<div className="sm:hover:!bg-blue-500" />"#, Some(json!([{"position": "legacy"}]))),
        (r#"<div className="!w-[100px]" />"#, Some(json!([{"position": "legacy"}]))),
        // Mixed with non-important classes
        (r#"<div className="w-4! hover:text-red-500! normal-class" />"#, None),
        (
            r#"<div className="!w-4 hover:!text-red-500 normal-class" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        // Empty or no className
        (r"<div />", None),
        (r#"<div className="" />"#, None),
        // Other attributes
        (r#"<div id="!important" />"#, None),
        (r#"<div data-test="!value" />"#, None),
    ];

    let fail = vec![
        // Should use recommended position (default)
        (r#"<div className="!w-4" />"#, None),
        (r#"<div className="hover:!text-red-500" />"#, None),
        (r#"<div className="sm:hover:!bg-blue-500" />"#, None),
        (r#"<div className="!w-[100px]" />"#, None),
        // Mixed inconsistent positions with recommended
        (r#"<div className="!w-4 text-red-500!" />"#, None),
        (r#"<div className="hover:!text-red-500 sm:bg-blue-500!" />"#, None),
        // Should use legacy position with option
        (r#"<div className="w-4!" />"#, Some(json!([{"position": "legacy"}]))),
        (r#"<div className="hover:text-red-500!" />"#, Some(json!([{"position": "legacy"}]))),
        (r#"<div className="sm:hover:bg-blue-500!" />"#, Some(json!([{"position": "legacy"}]))),
        (r#"<div className="w-[100px]!" />"#, Some(json!([{"position": "legacy"}]))),
        // Mixed inconsistent positions with legacy
        (r#"<div className="w-4! !text-red-500" />"#, Some(json!([{"position": "legacy"}]))),
        (
            r#"<div className="hover:text-red-500! sm:!bg-blue-500" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        // Multiple classes with wrong positions
        (r#"<div className="!w-4 !h-4 !bg-red-500" />"#, None),
        (r#"<div className="w-4! h-4! bg-red-500!" />"#, Some(json!([{"position": "legacy"}]))),
        // With normal classes mixed in
        (r#"<div className="!w-4 normal-class hover:!text-red-500" />"#, None),
        (
            r#"<div className="w-4! normal-class hover:text-red-500!" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
    ];

    let fix = vec![
        // Fix to recommended position (default)
        (r#"<div className="!w-4" />"#, r#"<div className="w-4!" />"#, None),
        (
            r#"<div className="hover:!text-red-500" />"#,
            r#"<div className="hover:text-red-500!" />"#,
            None,
        ),
        (
            r#"<div className="sm:hover:!bg-blue-500" />"#,
            r#"<div className="sm:hover:bg-blue-500!" />"#,
            None,
        ),
        (r#"<div className="!w-[100px]" />"#, r#"<div className="w-[100px]!" />"#, None),
        // Fix multiple classes to recommended
        (r#"<div className="!w-4 !h-4" />"#, r#"<div className="w-4! h-4!" />"#, None),
        (
            r#"<div className="!w-4 normal-class" />"#,
            r#"<div className="w-4! normal-class" />"#,
            None,
        ),
        // Fix to legacy position with option
        (
            r#"<div className="w-4!" />"#,
            r#"<div className="!w-4" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        (
            r#"<div className="hover:text-red-500!" />"#,
            r#"<div className="hover:!text-red-500" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        (
            r#"<div className="sm:hover:bg-blue-500!" />"#,
            r#"<div className="sm:hover:!bg-blue-500" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        // Fix multiple classes to legacy
        (
            r#"<div className="w-4! h-4!" />"#,
            r#"<div className="!w-4 !h-4" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        (
            r#"<div className="w-4! normal-class" />"#,
            r#"<div className="!w-4 normal-class" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
        // Fix mixed positions
        (
            r#"<div className="!w-4 text-red-500!" />"#,
            r#"<div className="w-4! text-red-500!" />"#,
            None,
        ),
        (
            r#"<div className="w-4! !text-red-500" />"#,
            r#"<div className="!w-4 !text-red-500" />"#,
            Some(json!([{"position": "legacy"}])),
        ),
    ];

    Tester::new(
        EnforceConsistentImportantPosition::NAME,
        EnforceConsistentImportantPosition::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
