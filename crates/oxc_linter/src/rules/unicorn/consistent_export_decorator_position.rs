use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn consistent_export_decorator_position_diagnostic(
    span: Span,
    expected_position: &str,
    actual_position: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Expected export decorators to be positioned `{expected_position}`, but found `{actual_position}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ConsistentExportDecoratorPositionOption {
    #[default]
    /// Require decorators on the line above the export.
    Above,
    /// Require decorators before export on the same line.
    Before,
    /// Require decorators after export or export default.
    After,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentExportDecoratorPosition(ConsistentExportDecoratorPositionOption);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent position for decorators applied to an exported class,
    /// relative to the `export` (or `export default`) keyword.
    ///
    /// ### Why is this bad?
    ///
    /// Decorators on an exported class can be written on the line above the export,
    /// directly before it, or directly after it. All three are valid syntax, but mixing
    /// styles across a codebase makes decorated classes harder to scan. This rule picks
    /// one consistent style and enforces it everywhere.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with default `"above"` option:
    /// ```js
    /// export default @decorator class Foo {}
    ///
    /// @decorator export default class Foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with default `"above"` option:
    /// ```js
    /// @decorator
    /// export default class Foo {}
    ///
    /// @foo
    /// @bar(options)
    /// export class Foo {}
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `"before"` option:
    /// ```js
    /// @decorator
    /// export default class Foo {}
    ///
    /// export default @decorator class Foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with `"before"` option:
    /// ```js
    /// @decorator export default class Foo {}
    ///
    /// @foo @bar(options) export default class Foo {}
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `"after"` option:
    /// ```js
    /// @decorator
    /// export default class Foo {}
    ///
    /// @decorator export default class Foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with `"after"` option:
    /// ```js
    /// export default @decorator class Foo {}
    ///
    /// export default @foo @bar(options) class Foo {}
    /// ```
    ConsistentExportDecoratorPosition,
    unicorn,
    style,
    pending,
    config = ConsistentExportDecoratorPositionOption,
    version = "next",
    short_description = "Enforce consistent decorator position on exported classes.",
);

impl Rule for ConsistentExportDecoratorPosition {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        if class.decorators.is_empty() {
            return;
        }

        let parent_node = ctx.nodes().parent_node(class.node_id());

        let (AstKind::ExportNamedDeclaration(_) | AstKind::ExportDefaultDeclaration(_)) =
            parent_node.kind()
        else {
            return;
        };

        let expected_position = match self.0 {
            ConsistentExportDecoratorPositionOption::Above => DecoratorPosition::Above,
            ConsistentExportDecoratorPositionOption::Before => DecoratorPosition::Before,
            ConsistentExportDecoratorPositionOption::After => DecoratorPosition::After,
        };

        let mut found_positions: FxHashMap<&str, u32> = FxHashMap::default();

        for decorator in &class.decorators {
            let actual_position = get_decorator_position(decorator.span(), parent_node.span(), ctx);
            *found_positions.entry(actual_position.as_str()).or_insert(0) += 1;
        }

        let actual_position = if found_positions.len() > 1 {
            "mixed"
        } else {
            found_positions.keys().next().copied().unwrap_or("")
        };

        if actual_position != expected_position.as_str() {
            ctx.diagnostic(consistent_export_decorator_position_diagnostic(
                // Match upstream: always report on the first decorator, even though
                // pointing at the actually offending decorator would arguably be more accurate.
                class.decorators[0].span(),
                expected_position.as_str(),
                actual_position,
            ));
        }
    }
}

#[derive(Clone, Copy)]
enum DecoratorPosition {
    Above,
    Before,
    After,
}

impl DecoratorPosition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Above => "above",
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

fn get_decorator_position(
    decorator_span: Span,
    parent_span: Span,
    ctx: &LintContext,
) -> DecoratorPosition {
    if decorator_span.start < parent_span.start {
        if has_blank_line_between(decorator_span, parent_span, ctx) {
            DecoratorPosition::Above
        } else {
            DecoratorPosition::Before
        }
    } else {
        DecoratorPosition::After
    }
}

fn has_blank_line_between(left: Span, right: Span, ctx: &LintContext) -> bool {
    if left.end >= right.start {
        return false;
    }

    let between_span = Span::new(left.end, right.start);

    ctx.source_range(between_span).contains("\n")
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("@decorator\nexport default class Foo {}", None),
        ("@decorator\nexport default class {}", None),
        ("@foo\n@bar(options)\nexport default class Foo {}", None),
        ("@decorator\nexport class Foo {}", None),
        ("@foo\n@bar(options)\nexport class Foo {}", None),
        (
            r"@decorator(
                foo
            )
            export class Foo {}",
            None,
        ),
        ("@decorator export default class Foo {}", Some(json!(["before"]))),
        ("@foo @bar(options) export default class Foo {}", Some(json!(["before"]))),
        ("@decorator export class Foo {}", Some(json!(["before"]))),
        (
            r"@decorator(
                foo
            ) export class Foo {}",
            Some(json!(["before"])),
        ),
        ("export default @decorator class Foo {}", Some(json!(["after"]))),
        ("export default @foo @bar(options) class Foo {}", Some(json!(["after"]))),
        ("export @decorator class Foo {}", Some(json!(["after"]))),
        ("class Foo {}", None),
        ("@decorator\nclass Foo {}", None),
        ("export default class Foo {}", None),
        ("export class Foo {}", None),
        ("export default (@decorator class Foo {})", None),
        ("export default class Foo { @decorator method() {} }", None),
        ("export {Foo}", None),
    ];

    let fail = vec![
        ("export default @decorator class Foo {}", None),
        ("export default @decorator class {}", None),
        ("@decorator export default class Foo {}", None),
        ("export @decorator class Foo {}", None),
        ("@decorator export class Foo {}", None),
        ("export default\n@decorator\nclass Foo {}", None),
        ("export\n@decorator\nclass Foo {}", None),
        ("export default @foo @bar(options) class Foo {}", None),
        ("@foo @bar(options) export default class Foo {}", None),
        ("@foo\n@bar export class Foo {}", None),
        // `@foo export @bar class Foo {}` is a parser-level syntax error in oxc (decorators
        // may not appear both before and after `export`/`export default`; see
        // `decorators_in_export_and_class` in crates/oxc_parser/src/js/module.rs). When the
        // parser reports any diagnostic, oxc_linter's service layer skips semantic analysis
        // and never runs lint rules for that file (crates/oxc_linter/src/service/runtime.rs),
        // so this rule's diagnostic can never take priority over the parser error. Left
        // commented out for reviewers to decide whether these cases should be removed.
        // ("@foo export @bar class Foo {}", None),
        // ("@foo export default @bar class Foo {}", None),
        (
            r"@decorator(
                foo
            ) export class Foo {}",
            None,
        ),
        (
            r"namespace N {
                @decorator export class Foo {}
            }",
            None,
        ),
        ("@decorator\nexport default class Foo {}", Some(json!(["before"]))),
        ("@foo\n@bar export class Foo {}", Some(json!(["before"]))),
        // Same parser-level syntax error as above; see the comment above the `None`-config
        // cases for details.
        // ("@foo export @bar class Foo {}", Some(json!(["before"]))),
        // ("@foo export default @bar class Foo {}", Some(json!(["before"]))),
        ("export default @decorator class Foo {}", Some(json!(["before"]))),
        ("@decorator\nexport class Foo {}", Some(json!(["before"]))),
        ("export @decorator class Foo {}", Some(json!(["before"]))),
        ("@decorator\nexport default class Foo {}", Some(json!(["after"]))),
        ("@decorator export default class Foo {}", Some(json!(["after"]))),
        ("@decorator\nexport class Foo {}", Some(json!(["after"]))),
        ("@decorator export class Foo {}", Some(json!(["after"]))),
        // Same parser-level syntax error as above; see the comment above the `None`-config
        // cases for details.
        // ("@foo export @bar class Foo {}", Some(json!(["after"]))),
        // ("@foo export default @bar class Foo {}", Some(json!(["after"]))),
        ("export default @decorator(/* comment */ value) class Foo {}", None),
        ("export default /* comment */ @decorator class Foo {}", None),
        ("@decorator /* comment */ export default class Foo {}", None),
        ("@decorator\nexport default abstract class Foo {}", Some(json!(["after"]))),
        (
            r"export default @decorator class Foo {
                method() {}
            }",
            None,
        ),
    ];

    Tester::new(
        ConsistentExportDecoratorPosition::NAME,
        ConsistentExportDecoratorPosition::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
