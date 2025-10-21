use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::class::ClassId;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn max_classes_per_file_diagnostic(total: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("File has too many classes ({total}). Maximum allowed is {max}",))
        .with_help("Reduce the number of classes in this file")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MaxClassesPerFile(Box<MaxClassesPerFileConfig>);

#[derive(Debug, Clone)]
pub struct MaxClassesPerFileConfig {
    pub max: usize,
    pub ignore_expressions: bool,
}

impl std::ops::Deref for MaxClassesPerFile {
    type Target = MaxClassesPerFileConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxClassesPerFileConfig {
    fn default() -> Self {
        Self { max: 1, ignore_expressions: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum number of classes per file
    ///
    /// ### Why is this bad?
    ///
    /// Files containing multiple classes can often result in a less navigable and poorly
    /// structured codebase. Best practice is to keep each file limited to a single responsibility.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class Foo {}
    /// class Bar {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function foo() {
    ///     var bar = 1;
    ///     let baz = 2;
    ///     const qux = 3;
    /// }
    /// ```
    MaxClassesPerFile,
    eslint,
    pedantic,
);

impl Rule for MaxClassesPerFile {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Self(Box::new(MaxClassesPerFileConfig { max, ignore_expressions: false }))
        } else {
            let max = value
                .get(0)
                .and_then(|config| config.get("max"))
                .and_then(serde_json::Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(1, |v| usize::try_from(v).unwrap_or(1));

            let ignore_expressions = value
                .get(0)
                .and_then(|config| config.get("ignoreExpressions"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            Self(Box::new(MaxClassesPerFileConfig { max, ignore_expressions }))
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut class_count = ctx.classes().declarations.len();

        if self.ignore_expressions {
            let class_expressions = ctx
                .classes()
                .iter_enumerated()
                .filter(|(_class_id, node_id)| !ctx.nodes().kind(**node_id).is_declaration())
                .count();
            class_count -= class_expressions;
        }

        if class_count <= self.max {
            return;
        }

        let node_id = ctx.classes().get_node_id(ClassId::from(self.max));
        let span = if let AstKind::Class(class) = ctx.nodes().kind(node_id) {
            class.span
        } else {
            Span::new(0, 0)
        };

        ctx.diagnostic(max_classes_per_file_diagnostic(class_count, self.max, span));
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.semantic().classes().len() > 0
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("class Foo {}", None),
        ("var x = class {};", None),
        ("var x = 5;", None),
        ("class Foo {}", Some(serde_json::json!([1]))),
        (
            "class Foo {}
			class Bar {}",
            Some(serde_json::json!([2])),
        ),
        ("class Foo {}", Some(serde_json::json!([{ "max": 1 }]))),
        (
            "class Foo {}
			class Bar {}",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "
			                class Foo {}
			                const myExpression = class {}
			            ",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 1 }])),
        ),
        (
            "
			                class Foo {}
			                class Bar {}
			                const myExpression = class {}
			            ",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 2 }])),
        ),
    ];

    let fail = vec![
        (
            "class Foo {}
			class Bar {}",
            None,
        ),
        (
            "class Foo {}
			const myExpression = class {}",
            None,
        ),
        (
            "var x = class {};
			var y = class {};",
            None,
        ),
        (
            "class Foo {}
			var x = class {};",
            None,
        ),
        ("class Foo {} class Bar {}", Some(serde_json::json!([1]))),
        ("class Foo {} class Bar {} class Baz {}", Some(serde_json::json!([2]))),
        (
            "
			                class Foo {}
			                class Bar {}
			                const myExpression = class {}
			            ",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 1 }])),
        ),
        (
            "
			                class Foo {}
			                class Bar {}
			                class Baz {}
			                const myExpression = class {}
			            ",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 2 }])),
        ),
    ];

    Tester::new(MaxClassesPerFile::NAME, MaxClassesPerFile::PLUGIN, pass, fail).test_and_snapshot();
}
