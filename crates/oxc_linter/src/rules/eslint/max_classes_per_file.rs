use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::class::ClassId;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn max_classes_per_file_diagnostic(total: u32, max: u32, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("File has too many classes ({total}). Maximum allowed is {max}"))
        .with_help("Reduce the number of classes in this file")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MaxClassesPerFile(Box<MaxClassesPerFileConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MaxClassesPerFileConfig {
    /// The maximum number of classes allowed per file.
    pub max: u32,
    /// Whether to ignore class expressions when counting classes.
    pub ignore_expressions: bool,
}

#[cfg(feature = "ruledocs")]
impl MaxClassesPerFile {
    #[expect(clippy::unnecessary_wraps)]
    pub fn config_schema(
        r#gen: &mut schemars::r#gen::SchemaGenerator,
    ) -> Option<schemars::schema::Schema> {
        let mut schema = r#gen.subschema_for::<MaxClassesPerFileConfig>();
        crate::utils::number_as_object_schema(r#gen, &mut schema, None);
        Some(schema)
    }
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
    /// Enforce a maximum number of classes per file.
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
    config = MaxClassesPerFileConfig,
    version = "0.3.4",
    short_description = "Enforce a maximum number of classes per file.",
);

impl Rule for MaxClassesPerFile {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        // if it's a number, treat it as the max value
        if let Some(max) = value
            .get(0)
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| u32::try_from(v).ok())
        {
            Ok(Self(Box::new(MaxClassesPerFileConfig { max, ignore_expressions: false })))
        } else {
            serde_json::from_value::<DefaultRuleConfig<Self>>(value)
                .map(DefaultRuleConfig::into_inner)
        }
    }

    #[expect(clippy::cast_possible_truncation)] // the count of classes can't be over u32::MAX, because the source code is already limited by u32::MAX.
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut class_count = ctx.classes().declarations.len() as u32;

        if self.ignore_expressions {
            let class_expressions = ctx
                .classes()
                .iter_enumerated()
                .filter(|(_class_id, node_id)| !ctx.nodes().kind(**node_id).is_declaration())
                .count() as u32;
            class_count -= class_expressions;
        }

        if class_count <= self.max {
            return;
        }

        let node_id = ctx.classes().get_node_id(ClassId::new(self.max as usize));
        let span = if let AstKind::Class(class) = ctx.nodes().kind(node_id) {
            class.span
        } else {
            Span::new(0, 0)
        };

        ctx.diagnostic(max_classes_per_file_diagnostic(class_count, self.max, span));
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        let classes = ctx.semantic().classes();
        let max = usize::try_from(self.max).unwrap_or(usize::MAX);
        if self.ignore_expressions {
            return classes.declarations.len() > max;
        }

        classes.len() > max
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
                const myExpression = class {}",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 1 }])),
        ),
        (
            "
                class Foo {}
                class Bar {}
                const myExpression = class {}",
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
                const myExpression = class {}",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 1 }])),
        ),
        (
            "
                class Foo {}
                class Bar {}
                class Baz {}
                const myExpression = class {}",
            Some(serde_json::json!([{ "ignoreExpressions": true, "max": 2 }])),
        ),
    ];

    Tester::new(MaxClassesPerFile::NAME, MaxClassesPerFile::PLUGIN, pass, fail).test_and_snapshot();
}
