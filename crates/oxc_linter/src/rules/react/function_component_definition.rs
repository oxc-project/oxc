use oxc_ast::{AstKind, ast::FunctionType};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{function_body_contains_jsx, function_contains_jsx},
};

fn function_component_definition_diagnostic(span: Span, expected: FunctionStyle) -> OxcDiagnostic {
    let article = if expected == FunctionStyle::Arrow { "an" } else { "a" };
    OxcDiagnostic::warn(format!("Function component is not {article} {}", expected.description()))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct FunctionComponentDefinitionConfig {
    named_components: NamedComponents,
    unnamed_components: UnnamedComponents,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum NamedComponents {
    Single(NamedComponentStyle),
    Multiple(Vec<NamedComponentStyle>),
}

impl Default for NamedComponents {
    fn default() -> Self {
        Self::Single(NamedComponentStyle::FunctionDeclaration)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum NamedComponentStyle {
    #[default]
    FunctionDeclaration,
    ArrowFunction,
    FunctionExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum UnnamedComponents {
    Single(UnnamedComponentStyle),
    Multiple(Vec<UnnamedComponentStyle>),
}

impl Default for UnnamedComponents {
    fn default() -> Self {
        Self::Single(UnnamedComponentStyle::FunctionExpression)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum UnnamedComponentStyle {
    ArrowFunction,
    #[default]
    FunctionExpression,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FunctionComponentDefinition(Box<FunctionComponentDefinitionConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent function form for React function components.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing declarations, function expressions, and arrow functions makes component definitions
    /// less predictable and harder to scan.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const Component = () => <div />;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component() { return <div />; }
    /// ```
    FunctionComponentDefinition,
    react,
    style,
    conditional_suggestion,
    config = FunctionComponentDefinition,
    version = "next",
    short_description = "Enforce a specific function type for function components.",
);

impl Rule for FunctionComponentDefinition {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (actual, is_component) = match node.kind() {
            AstKind::Function(function) => {
                let actual = match function.r#type {
                    FunctionType::FunctionDeclaration => FunctionStyle::Declaration,
                    FunctionType::FunctionExpression => FunctionStyle::Expression,
                    _ => return,
                };

                (actual, function_contains_jsx(function))
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                (FunctionStyle::Arrow, function_body_contains_jsx(&arrow.body))
            }
            _ => return,
        };
        if !is_component || matches!(ctx.nodes().parent_kind(node.id()), AstKind::ObjectProperty(_))
        {
            return;
        }

        let named = is_named(node, ctx);
        let (allowed, expected) = if named {
            (self.0.named_components.allows(actual), self.0.named_components.preferred())
        } else {
            (self.0.unnamed_components.allows(actual), self.0.unnamed_components.preferred())
        };
        if allowed {
            return;
        }

        let diagnostic = function_component_definition_diagnostic(node.span(), expected);
        if fix::can_fix(node, ctx, expected) {
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                let (span, replacement) = fix::replacement(node, ctx, expected, named);
                fixer.replace(span, replacement)
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionStyle {
    Declaration,
    Expression,
    Arrow,
}

impl FunctionStyle {
    fn description(self) -> &'static str {
        match self {
            Self::Declaration => "function declaration",
            Self::Expression => "function expression",
            Self::Arrow => "arrow function",
        }
    }
}

impl NamedComponents {
    fn preferred(&self) -> FunctionStyle {
        match self {
            Self::Single(style) => style.style(),
            Self::Multiple(styles) => {
                styles.first().map_or(FunctionStyle::Declaration, |style| style.style())
            }
        }
    }

    fn allows(&self, style: FunctionStyle) -> bool {
        match self {
            Self::Single(item) => item.style() == style,
            Self::Multiple(styles) => styles.iter().any(|item| item.style() == style),
        }
    }
}

impl NamedComponentStyle {
    fn style(self) -> FunctionStyle {
        match self {
            Self::FunctionDeclaration => FunctionStyle::Declaration,
            Self::FunctionExpression => FunctionStyle::Expression,
            Self::ArrowFunction => FunctionStyle::Arrow,
        }
    }
}

impl UnnamedComponents {
    fn preferred(&self) -> FunctionStyle {
        match self {
            Self::Single(style) => style.style(),
            Self::Multiple(styles) => {
                styles.first().map_or(FunctionStyle::Expression, |style| style.style())
            }
        }
    }

    fn allows(&self, style: FunctionStyle) -> bool {
        match self {
            Self::Single(item) => item.style() == style,
            Self::Multiple(styles) => styles.iter().any(|item| item.style() == style),
        }
    }
}

impl UnnamedComponentStyle {
    fn style(self) -> FunctionStyle {
        match self {
            Self::FunctionExpression => FunctionStyle::Expression,
            Self::ArrowFunction => FunctionStyle::Arrow,
        }
    }
}

fn is_named(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    matches!(node.kind(), AstKind::Function(function) if function.r#type == FunctionType::FunctionDeclaration)
        || matches!(ctx.nodes().parent_kind(node.id()), AstKind::VariableDeclarator(_))
}

mod fix {
    use oxc_ast::{
        AstKind,
        ast::{
            ArrowFunctionExpression, FormalParameters, Function, FunctionType,
            TSTypeParameterDeclaration, VariableDeclarator,
        },
    };
    use oxc_span::{GetSpan, Span};

    use super::FunctionStyle;
    use crate::{AstNode, context::LintContext};

    pub(super) fn can_fix<'a>(
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        expected: FunctionStyle,
    ) -> bool {
        match node.kind() {
            AstKind::Function(function) => {
                if function.r#type == FunctionType::FunctionDeclaration
                    && matches!(
                        ctx.nodes().parent_kind(node.id()),
                        AstKind::ExportDefaultDeclaration(_)
                    )
                {
                    return false;
                }
                if function.r#type == FunctionType::FunctionExpression && function.id.is_some() {
                    return false;
                }
                if expected == FunctionStyle::Declaration
                    && variable_declarator(node, ctx)
                        .is_some_and(|declaration| declaration.type_annotation.is_some())
                {
                    return false;
                }
                if expected == FunctionStyle::Arrow
                    && (function.generator
                        || has_one_unconstrained_type_parameter(
                            function.type_parameters.as_deref(),
                        ))
                {
                    return false;
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if expected == FunctionStyle::Declaration
                    && variable_declarator(node, ctx)
                        .is_some_and(|declaration| declaration.type_annotation.is_some())
                {
                    return false;
                }
                if expected == FunctionStyle::Arrow
                    && has_one_unconstrained_type_parameter(arrow.type_parameters.as_deref())
                {
                    return false;
                }
            }
            _ => return false,
        }
        true
    }

    fn has_one_unconstrained_type_parameter(
        parameters: Option<&TSTypeParameterDeclaration<'_>>,
    ) -> bool {
        parameters.is_some_and(|parameters| {
            parameters.params.len() == 1 && parameters.params[0].constraint.is_none()
        })
    }

    fn variable_declarator<'a, 'c>(
        node: &AstNode<'a>,
        ctx: &'c LintContext<'a>,
    ) -> Option<&'c VariableDeclarator<'a>> {
        match ctx.nodes().parent_kind(node.id()) {
            AstKind::VariableDeclarator(declaration) => Some(declaration),
            _ => None,
        }
    }

    pub(super) fn replacement<'a>(
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        expected: FunctionStyle,
        named: bool,
    ) -> (Span, String) {
        let parts = FunctionParts::new(node, ctx);
        let body = if parts.expression_body {
            format!("{{\n return {}\n}}", parts.body)
        } else {
            parts.body.clone()
        };
        let function = match (expected, named) {
            (FunctionStyle::Declaration, true) => format!(
                "{}function{} {}{}{}{} {}",
                parts.async_prefix,
                parts.generator_marker,
                parts.name,
                parts.type_parameters,
                parts.params,
                parts.return_type,
                body
            ),
            (FunctionStyle::Expression, true) => format!(
                "{} {}{} = {}function{}{}{}{} {}",
                parts.variable_kind,
                parts.name,
                parts.type_annotation,
                parts.async_prefix,
                parts.generator_marker,
                parts.type_parameters,
                parts.params,
                parts.return_type,
                body
            ),
            (FunctionStyle::Arrow, true) => format!(
                "{} {}{} = {}{}{}{} => {}",
                parts.variable_kind,
                parts.name,
                parts.type_annotation,
                parts.async_prefix,
                parts.type_parameters,
                parts.params,
                parts.return_type,
                body
            ),
            (FunctionStyle::Expression, false) => format!(
                "{}function{}{}{}{} {}",
                parts.async_prefix,
                parts.generator_marker,
                parts.type_parameters,
                parts.params,
                parts.return_type,
                body
            ),
            (FunctionStyle::Arrow, false) => format!(
                "{}{}{}{} => {}",
                parts.async_prefix, parts.type_parameters, parts.params, parts.return_type, body
            ),
            (FunctionStyle::Declaration, false) => unreachable!(),
        };
        (parts.replace_span, function)
    }

    struct FunctionParts<'a> {
        replace_span: Span,
        name: String,
        variable_kind: &'static str,
        type_annotation: &'a str,
        async_prefix: &'static str,
        generator_marker: &'static str,
        type_parameters: String,
        params: String,
        return_type: String,
        body: String,
        expression_body: bool,
    }

    impl<'a> FunctionParts<'a> {
        fn new(node: &AstNode<'a>, ctx: &LintContext<'a>) -> Self {
            match node.kind() {
                AstKind::Function(function) => Self::from_function(function, node, ctx),
                AstKind::ArrowFunctionExpression(arrow) => Self::from_arrow(arrow, node, ctx),
                _ => unreachable!(),
            }
        }

        fn from_function(
            function: &Function<'a>,
            node: &AstNode<'a>,
            ctx: &LintContext<'a>,
        ) -> Self {
            let declaration = variable_declarator(node, ctx);
            let (replace_span, variable_kind) = variable_context(node, declaration, ctx);
            Self {
                replace_span,
                name: function.id.as_ref().map_or_else(
                    || variable_name(declaration),
                    |identifier| identifier.name.to_string(),
                ),
                variable_kind,
                type_annotation: type_annotation(declaration, ctx),
                async_prefix: if function.r#async { "async " } else { "" },
                generator_marker: if function.generator { "*" } else { "" },
                type_parameters: function
                    .type_parameters
                    .as_ref()
                    .map_or_else(String::new, |item| ctx.source_range(item.span).to_string()),
                // Function parameter spans always include their parentheses.
                params: ctx.source_range(function.params.span).to_string(),
                return_type: function
                    .return_type
                    .as_ref()
                    .map_or_else(String::new, |item| ctx.source_range(item.span).to_string()),
                body: function
                    .body
                    .as_ref()
                    .map_or_else(String::new, |body| ctx.source_range(body.span).to_string()),
                expression_body: false,
            }
        }

        fn from_arrow(
            arrow: &ArrowFunctionExpression<'a>,
            node: &AstNode<'a>,
            ctx: &LintContext<'a>,
        ) -> Self {
            let declaration = variable_declarator(node, ctx);
            let (replace_span, variable_kind) = variable_context(node, declaration, ctx);
            Self {
                replace_span,
                name: variable_name(declaration),
                variable_kind,
                type_annotation: type_annotation(declaration, ctx),
                async_prefix: if arrow.r#async { "async " } else { "" },
                generator_marker: "",
                type_parameters: arrow
                    .type_parameters
                    .as_ref()
                    .map_or_else(String::new, |item| ctx.source_range(item.span).to_string()),
                params: parenthesized_arrow_params(&arrow.params, ctx),
                return_type: arrow
                    .return_type
                    .as_ref()
                    .map_or_else(String::new, |item| ctx.source_range(item.span).to_string()),
                body: ctx.source_range(arrow.body.span).to_string(),
                expression_body: arrow.expression,
            }
        }
    }

    fn variable_context<'a>(
        node: &AstNode<'a>,
        declaration: Option<&VariableDeclarator<'a>>,
        ctx: &LintContext<'a>,
    ) -> (Span, &'static str) {
        let Some(declaration) = declaration else {
            return (node.span(), "const");
        };
        let AstKind::VariableDeclaration(variable) = ctx.nodes().parent_kind(declaration.node_id())
        else {
            unreachable!()
        };
        (variable.span, variable.kind.as_str())
    }

    fn variable_name(declaration: Option<&VariableDeclarator<'_>>) -> String {
        declaration
            .and_then(|declaration| declaration.id.get_identifier_name())
            .map_or_else(String::new, |name| name.to_string())
    }

    fn type_annotation<'a>(
        declaration: Option<&VariableDeclarator<'_>>,
        ctx: &LintContext<'a>,
    ) -> &'a str {
        declaration
            .and_then(|declaration| declaration.type_annotation.as_ref())
            .map_or("", |annotation| ctx.source_range(annotation.span))
    }

    fn parenthesized_arrow_params(params: &FormalParameters<'_>, ctx: &LintContext<'_>) -> String {
        let source = ctx.source_range(params.span);
        // Simple arrow parameter spans omit parentheses; parenthesized arrow spans include them.
        if source.starts_with('(') { source.to_string() } else { format!("({source})") }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                    class Hello extends React.Component {
                      render() { return <div>Hello {this.props.name}</div> }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    class Hello extends React.Component {
                      render() { return <div>Hello {this.props.name}</div> }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    class Hello extends React.Component {
                      render() { return <div>Hello {this.props.name}</div> }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "var Hello = (props) => { return <div/> }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "const Hello = (props) => { return <div/> }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "function Hello(props) { return <div/> }",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "var Hello = function(props) { return <div/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "const Hello = function(props) { return <div/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "function Hello() { return function() { return <div/> } }",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "function Hello() { return () => { return <div/> }}",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "var Foo = React.memo(function Foo() { return <p/> })",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "const Foo = React.memo(function Foo() { return <p/> })",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    const selectAvatarByUserId = (state, id) => {
                      const user = selectUserById(state, id)
                      return null
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function ensureValidSourceType(sourceType: string) {
                      switch (sourceType) {
                        case 'ALBUM':
                        case 'PLAYLIST':
                          return sourceType;
                        default:
                          return null;
                      }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "function Hello(props: Test) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "var Hello = function(props: Test) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "var Hello = (props: Test) => { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "var Hello: React.FC<Test> = function(props) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "var Hello: React.FC<Test> = (props) => { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "function Hello<Test>(props: Props<Test>) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "function Hello<Test extends {}>(props: Props<Test>) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "var Hello = function<Test>(props: Props<Test>) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "var Hello = function<Test extends {}>(props: Props<Test>) { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "var Hello = <Test extends {}>(props: Props<Test>) => { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "function wrapper() { return function<Test>(props: Props<Test>) { return <p/> } } ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "function wrapper() { return function<Test extends {}>(props: Props<Test>) { return <p/> } } ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "function wrapper() { return<Test extends {}>(props: Props<Test>) => { return <p/> } } ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "var Hello = function(props): ReactNode { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "var Hello = (props): ReactNode => { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "function wrapper() { return function(props): ReactNode { return <p/> } }",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "function wrapper() { return (props): ReactNode => { return <p/> } }",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "function Hello(props): ReactNode { return <p/> }",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    const obj = {
                      serialize: (el) => {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    const obj = {
                      serialize: (el) => {
                        return <p/>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    const obj = {
                      serialize: (el) => {
                        return <p/>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    const obj = {
                      serialize: function (el) {
                        return <p/>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    const obj = {
                      serialize: function (el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    const obj = {
                      serialize: function (el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    const obj = {
                      serialize(el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    const obj = {
                      serialize(el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    const obj = {
                      serialize(el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    const obj = {
                      serialize(el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    const obj = {
                      serialize(el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    const obj = {
                      serialize: (el) => {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    const obj = {
                      serialize: (el) => {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    const obj = {
                      serialize: function (el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    const obj = {
                      serialize: function (el) {
                        return <p/>
                      }
                    };
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "function Hello(props) { return <div/> }",
            Some(
                serde_json::json!([{ "namedComponents": ["function-declaration", "function-expression"] }]),
            ),
        ),
        (
            "var Hello = function(props) { return <div/> }",
            Some(
                serde_json::json!([{ "namedComponents": ["function-declaration", "function-expression"] }]),
            ),
        ),
        (
            "var Foo = React.memo(function Foo() { return <p/> })",
            Some(
                serde_json::json!([{ "namedComponents": ["function-declaration", "function-expression"] }]),
            ),
        ),
        (
            "function Hello(props: Test) { return <p/> }",
            Some(
                serde_json::json!([{ "namedComponents": ["function-declaration", "function-expression"] }]),
            ),
        ),
        (
            "var Hello = function(props: Test) { return <p/> }",
            Some(
                serde_json::json!([{ "namedComponents": ["function-expression", "function-expression"] }]),
            ),
        ),
        (
            "var Hello = (props: Test) => { return <p/> }",
            Some(
                serde_json::json!([{ "namedComponents": ["arrow-function", "function-expression"] }]),
            ),
        ),
        (
            "
                    function wrap(Component) {
                      return function(props) {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            Some(
                serde_json::json!([{ "unnamedComponents": ["arrow-function", "function-expression"] }]),
            ),
        ),
        (
            "
                    function wrap(Component) {
                      return (props) => {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            Some(
                serde_json::json!([{ "unnamedComponents": ["arrow-function", "function-expression"] }]),
            ),
        ),
        (
            "
                    export default (key, subTree = {}) => {
                      return (state) => {
                        const dataInStore = getFromDataModel(key)(state);
                        const fullPaths = dataInStore.map((item, index) => {
                          return [key, index];
                        });
            
                        return {
                          key,
                          paths: fullPaths.map((p) => [p[1]]),
                          fullPaths,
                          subTree: Object.keys(subTree).length ? subTree : null,
                        }
                      };
                    }
                  ",
            None,
        ),
        (
            "
                    function mapStateToProps() {
                      const internItems = makeInternArray();
                      const internClassList = makeInternArray();
            
                      return (state, props) => {
                        const { store, bucket, singleCharacter } = props;
            
                        return {
                          store: null,
                          destinyVersion: store.destinyVersion,
                          storeId: store.id,
                        }
                      }
                    }
                  ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = function(props) {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = function(props) {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    let Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    let Hello;
                    Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    let Hello = (props) => {
                      return <div/>;
                    }
                    Hello = function(props) {
                      return <span/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function(props) {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return (props) => {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = (props: Test) => {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = function(props: Test) {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function Hello(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = function(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function Hello(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    import * as React from 'react';
                    function Hello(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    export function Hello(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function Hello(props) {
                      return React.createElement('div');
                    }
                    export default Hello;
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = (props: Test) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello: React.FC<Test> = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello: React.FC<Test> = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello: React.FC<Test> = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello: React.FC<Test> = (props) => {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function Hello<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello<Test>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    };
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = <Test extends {}>(props: Test) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function<Test extends {}>(props) {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function<Test>(props) {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return <Test extends {}>(props) => {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function(props): ReactNode {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return (props): ReactNode => {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    export function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    export var Hello = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    export var Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    export default function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    module.exports = function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(
                serde_json::json!([{ "namedComponents": ["arrow-function", "function-expression"] }]),
            ),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            Some(
                serde_json::json!([{ "namedComponents": ["function-declaration", "function-expression"] }]),
            ),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            Some(
                serde_json::json!([{ "namedComponents": ["function-expression", "function-declaration"] }]),
            ),
        ),
        (
            "
                    const genX = (symbol) => `the symbol is ${symbol}`;
            
                    const IndexPage = () => {
                      return (
                        <div>
                          Hello World.{genX('$')}
                        </div>
                      )
                    }
            
                    export default IndexPage;
                  ",
            Some(serde_json::json!([{ "namedComponents": ["function-declaration"] }])),
        ),
        (
            "function* Hello(props) { return <div/>; }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
    ];

    let fix = vec![
        (
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = function(props) {
                      return <div/>;
                    };
                  ",
            "
                    var Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = function(props) {
                      return <div/>;
                    };
                  ",
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            "
                    var Hello = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    let Hello = (props) => {
                      return <div/>;
                    }
                  ",
            "
                    let Hello = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    let Hello;
                    Hello = (props) => {
                      return <div/>;
                    }
                  ",
            "
                    let Hello;
                    Hello = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    let Hello = (props) => {
                      return <div/>;
                    }
                    Hello = function(props) {
                      return <span/>;
                    }
                  ",
            "
                    let Hello = function(props) {
                      return <div/>;
                    }
                    Hello = function(props) {
                      return <span/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function(props) {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            "
                    function wrap(Component) {
                      return (props) => {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return (props) => {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            "
                    function wrap(Component) {
                      return function(props) {
                        return <div><Component {...props}/></div>;
                      };
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = (props: Test) => {
                      return <div/>;
                    };
                  ",
            "
                    function Hello(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = function(props: Test) {
                      return <div/>;
                    };
                  ",
            "
                    function Hello(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function Hello(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = (props: Test) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = function(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    var Hello = (props: Test) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = function(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function Hello(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            "
                    const Hello = function(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    import * as React from 'react';
                    function Hello(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            "
                    import * as React from 'react';
                    const Hello = function(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    export function Hello(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            "
                    export const Hello = function(props: Test) {
                      return React.createElement('div');
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    function Hello(props) {
                      return React.createElement('div');
                    }
                    export default Hello;
                  ",
            "
                    const Hello = function(props) {
                      return React.createElement('div');
                    }
                    export default Hello;
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = (props: Test) => {
                      return <div/>;
                    }
                  ",
            "
                    var Hello = function(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello: React.FC<Test> = (props) => {
                      return <div/>;
                    }
                  ",
            "
                    var Hello: React.FC<Test> = function(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello: React.FC<Test> = function(props) {
                      return <div/>;
                    }
                  ",
            "
                    var Hello: React.FC<Test> = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = <Test extends {}>(props: Test) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    function Hello<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    };
                  ",
            "
                    function Hello<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    var Hello = <Test extends {}>(props: Test) => {
                      return <div/>;
                    }
                  ",
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    var Hello = <Test extends {}>(props: Test) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    var Hello = function<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            "
                    function Hello<Test extends {}>(props: Test) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function<Test extends {}>(props) {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            "
                    function wrap(Component) {
                      return <Test extends {}>(props) => {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return <Test extends {}>(props) => {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            "
                    function wrap(Component) {
                      return function<Test extends {}>(props) {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return function(props): ReactNode {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            "
                    function wrap(Component) {
                      return (props): ReactNode => {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "arrow-function" }])),
        ),
        (
            "
                    function wrap(Component) {
                      return (props): ReactNode => {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            "
                    function wrap(Component) {
                      return function(props): ReactNode {
                        return <div><Component {...props}/></div>
                      }
                    }
                  ",
            Some(serde_json::json!([{ "unnamedComponents": "function-expression" }])),
        ),
        (
            "
                    export function Hello(props) {
                      return <div/>;
                    }
                  ",
            "
                    export const Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    export var Hello = function(props) {
                      return <div/>;
                    }
                  ",
            "
                    export var Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "
                    export var Hello = (props) => {
                      return <div/>;
                    }
                  ",
            "
                    export function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            "
                    const Hello = (props) => {
                      return <div/>;
                    }
                  ",
            Some(
                serde_json::json!([{ "namedComponents": ["arrow-function", "function-expression"] }]),
            ),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            "
                    function Hello(props) {
                      return <div/>;
                    }
                  ",
            Some(
                serde_json::json!([{ "namedComponents": ["function-declaration", "function-expression"] }]),
            ),
        ),
        (
            "
                    var Hello = (props) => {
                      return <div/>;
                    };
                  ",
            "
                    var Hello = function(props) {
                      return <div/>;
                    }
                  ",
            Some(
                serde_json::json!([{ "namedComponents": ["function-expression", "function-declaration"] }]),
            ),
        ),
        (
            "
                    const genX = (symbol) => `the symbol is ${symbol}`;
            
                    const IndexPage = () => {
                      return (
                        <div>
                          Hello World.{genX('$')}
                        </div>
                      )
                    }
            
                    export default IndexPage;
                  ",
            "
                    const genX = (symbol) => `the symbol is ${symbol}`;
            
                    function IndexPage() {
                      return (
                        <div>
                          Hello World.{genX('$')}
                        </div>
                      )
                    }
            
                    export default IndexPage;
                  ",
            Some(serde_json::json!([{ "namedComponents": ["function-declaration"] }])),
        ),
        (
            "async function Hello(props) { await load(); return <div/>; }",
            "const Hello = async (props) => { await load(); return <div/>; }",
            Some(serde_json::json!([{ "namedComponents": "arrow-function" }])),
        ),
        (
            "const Hello = async (props) => { await load(); return <div/>; }",
            "async function Hello(props) { await load(); return <div/>; }",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
        (
            "async function* Hello(props) { yield await load(); return <div/>; }",
            "const Hello = async function*(props) { yield await load(); return <div/>; }",
            Some(serde_json::json!([{ "namedComponents": "function-expression" }])),
        ),
        (
            "const Hello = props => <div/>;",
            "function Hello(props) {\n return <div/>\n}",
            Some(serde_json::json!([{ "namedComponents": "function-declaration" }])),
        ),
    ];

    Tester::new(FunctionComponentDefinition::NAME, FunctionComponentDefinition::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
