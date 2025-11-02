use oxc_ast::{AstKind, ast::*};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{context::LintContext, rule::Rule};

const SCOPE: &str = "eslint-plugin-react-refresh";

fn export_all_components_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This rule can't verify that `export *` only exports components.")
        .with_label(span)
        .with_error_code_scope(SCOPE)
}

fn named_export_components_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file only exports components. Use a new file to share constants or functions between components.")
        .with_label(span)
        .with_error_code_scope(SCOPE)
}

fn anonymous_components_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Fast refresh can't handle anonymous components. Add a name to your export.",
    )
    .with_label(span)
    .with_error_code_scope(SCOPE)
}

fn local_components_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file only exports components. Move your component(s) to a separate file.")
        .with_label(span)
        .with_error_code_scope(SCOPE)
}

fn no_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file has exports. Move your component(s) to a separate file.")
        .with_label(span)
        .with_error_code_scope(SCOPE)
}

fn react_context_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file only exports components. Move your React context(s) to a separate file.")
        .with_label(span)
        .with_error_code_scope(SCOPE)
}

#[derive(Debug, Default, Clone)]
pub struct OnlyExportComponents(Box<OnlyExportComponentsConfig>);

impl std::ops::Deref for OnlyExportComponents {
    type Target = OnlyExportComponentsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct OnlyExportComponentsConfig {
    allow_export_names: FxHashSet<String>,
    allow_constant_export: bool,
    custom_hocs: Vec<String>,
    check_js: bool,
}

// NOTE: Ensure this is always kept in sync with OnlyExportComponentsConfig
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
struct OnlyExportComponentsOptionsJson {
    /// Treat specific named exports as HMR-safe (useful for frameworks that hot-replace
    /// certain exports). For example, in Remix:
    /// `{ "allowExportNames": ["meta", "links", "headers", "loader", "action"] }`
    #[serde(rename = "allowExportNames")]
    allow_export_names: Option<Vec<String>>,
    /// Allow exporting primitive constants (string/number/boolean/template literal)
    /// alongside component exports without triggering a violation. Recommended when your
    /// bundler’s Fast Refresh integration supports this (enabled by the plugin’s `vite`
    /// preset).
    ///
    /// ```jsx
    /// // Allowed when allowConstantExport: true
    /// export const VERSION = "3";
    /// export const Foo = () => null;
    /// ```
    #[serde(rename = "allowConstantExport")]
    allow_constant_export: Option<bool>,
    /// If you export components wrapped in custom higher-order components, list their
    /// identifiers here to avoid false positives.
    #[serde(rename = "customHOCs")]
    custom_hocs: Option<Vec<String>>,
    /// Check `.js` files that contain JSX (in addition to `.tsx`/`.jsx`). To reduce
    /// false positives, only files that import React are checked when this is enabled.
    #[serde(rename = "checkJS")]
    check_js: Option<bool>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that modules only **export React components (and related HMR-safe items)** so
    /// that Fast Refresh (a.k.a. hot reloading) can safely preserve component state.
    /// Concretely, it validates the shape of your module’s exports and common entrypoints
    /// (e.g. `createRoot(...).render(<App />)`) to match what integrations like
    /// `react-refresh` expect. The rule name is `react-refresh/only-export-components`.
    ///
    /// ### Why is this bad?
    ///
    /// Fast Refresh can only reliably retain state if a module exports components and
    /// avoids patterns that confuse the refresh runtime. Problematic patterns (like
    /// `export *`, anonymous default functions, exporting arrays of JSX, or mixing
    /// non-component exports in unsupported ways) can cause:
    ///
    /// - Components to remount and lose state on edit
    /// - Missed updates (no refresh) or overly broad reloads
    /// - Fragile HMR behavior that differs between bundlers
    ///
    /// By enforcing predictable exports, edits stay fast and stateful during development.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```jsx
    /// // 1) Mixing util exports with components in unsupported ways
    /// export const foo = () => {};      // util, not a component
    /// export const Bar = () => <></>;   // component
    /// ```
    ///
    /// ```jsx
    /// // 2) Anonymous default export (name is required)
    /// export default function () {}
    /// ```
    ///
    /// ```jsx
    /// // 3) Re-exporting everything hides what’s exported
    /// export * from "./foo";
    /// ```
    ///
    /// ```jsx
    /// // 4) Exporting JSX collections makes components non-discoverable
    /// const Tab = () => null;
    /// export const tabs = [<Tab />, <Tab />];
    /// ```
    ///
    /// ```jsx
    /// // 5) Bootstrapping a root within the same module that defines components
    /// const App = () => null;
    /// createRoot(document.getElementById("root")).render(<App />);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```jsx
    /// // Named or default component exports are fine
    /// export default function Foo() {
    ///   return null;
    /// }
    /// ```
    ///
    /// ```jsx
    /// // Utilities may coexist if allowed by options or naming conventions
    /// const foo = () => {};
    /// export const Bar = () => null;
    /// ```
    ///
    /// ```jsx
    /// // Entrypoint files may render an imported component
    /// import { App } from "./App";
    /// createRoot(document.getElementById("root")).render(<App />);
    /// ```
    OnlyExportComponents,
    react,
    restriction,
    config = OnlyExportComponentsOptionsJson,
);

static DEFAULT_REACT_HOCS: &[&str] = &["memo", "forwardRef"];

impl Rule for OnlyExportComponents {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|first| {
                serde_json::from_value::<OnlyExportComponentsOptionsJson>(first.clone()).ok()
            })
            .map(|config| OnlyExportComponentsConfig {
                allow_export_names: config
                    .allow_export_names
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                allow_constant_export: config.allow_constant_export.unwrap_or_default(),
                custom_hocs: config.custom_hocs.unwrap_or_default(),
                check_js: config.check_js.unwrap_or_default(),
            })
            .unwrap_or_default();

        Self(Box::new(config))
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        let Some(filename) = ctx.file_path().file_name().and_then(|s| s.to_str()) else {
            return false;
        };

        let should_scan = {
            let ext = ctx.file_extension();
            matches!(ext, Some(e) if e.eq_ignore_ascii_case("tsx") || e.eq_ignore_ascii_case("jsx"))
                || (self.check_js && matches!(ext, Some(e) if e.eq_ignore_ascii_case("js")))
        };

        let should_skip = filename.contains(".test.")
            || filename.contains(".spec.")
            || filename.contains(".cy.")
            || filename.contains(".stories.");

        should_scan && !should_skip
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let react_is_in_scope = ctx
            .module_record()
            .import_entries
            .iter()
            .any(|entry| entry.module_request.name == "react");
        if self.check_js && !react_is_in_scope {
            return;
        }

        let export_info = self.analyze_exports(ctx);
        let local_components = self.find_local_components(ctx);

        Self::report_diagnostics(ctx, &export_info, &local_components);
    }
}

impl OnlyExportComponents {
    fn is_react_hoc(&self, name: &str) -> bool {
        DEFAULT_REACT_HOCS.contains(&name) || self.custom_hocs.iter().any(|h| h.as_str() == name)
    }

    fn starts_with_ascii_upper(s: &str) -> bool {
        matches!(s.as_bytes().first(), Some(b'A'..=b'Z'))
    }

    fn can_be_react_function_component(&self, init: Option<&Expression>) -> bool {
        if let Some(raw_init) = init {
            let js_init = Self::skip_ts_expression(raw_init);

            match js_init {
                Expression::ArrowFunctionExpression(_) => true,
                Expression::CallExpression(call_expr) => {
                    if let Expression::Identifier(callee) = &call_expr.callee {
                        self.is_react_hoc(&callee.name)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn skip_ts_expression<'a>(exp: &'a Expression<'a>) -> &'a Expression<'a> {
        match exp {
            Expression::TSAsExpression(ts_expr) => &ts_expr.expression,
            Expression::TSSatisfiesExpression(ts_expr) => &ts_expr.expression,
            _ => exp,
        }
    }

    fn is_exported(ctx: &LintContext, node_id: NodeId) -> bool {
        let semantic = ctx.semantic();
        let nodes = semantic.nodes();

        std::iter::successors(Some(node_id), |&id| {
            let parent = nodes.parent_id(id);
            if parent == id { None } else { Some(parent) }
        })
        .any(|id| {
            matches!(
                nodes.get_node(id).kind(),
                AstKind::ExportDefaultDeclaration(_) | AstKind::ExportNamedDeclaration(_)
            )
        })
    }

    fn analyze_exports(&self, ctx: &LintContext) -> ExportAnalysis {
        let mut analysis = ExportAnalysis::default();
        let module_record = ctx.module_record();

        let has_any_exports = !module_record.local_export_entries.is_empty()
            || !module_record.star_export_entries.is_empty()
            || !module_record.indirect_export_entries.is_empty();
        if !has_any_exports {
            return analysis;
        }

        analysis.has_exports = true;

        for node in ctx.semantic().nodes() {
            match node.kind() {
                AstKind::ExportAllDeclaration(export_all) if export_all.export_kind.is_value() => {
                    ctx.diagnostic(export_all_components_diagnostic(export_all.span));
                }
                AstKind::ExportDefaultDeclaration(export_default) => {
                    let result = self.analyze_export_default(export_default);
                    if let Some(span) = result.anonymous_span {
                        ctx.diagnostic(anonymous_components_diagnostic(span));
                    }
                    analysis.merge(result);
                }
                AstKind::ExportNamedDeclaration(export_named)
                    if export_named.export_kind.is_value() =>
                {
                    let result = self.analyze_export_named(ctx, export_named);
                    analysis.merge(result);
                }
                _ => {}
            }
        }

        analysis
    }

    fn find_local_components(&self, ctx: &LintContext) -> Vec<Span> {
        ctx.semantic()
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::VariableDeclaration(var_decl) => {
                    var_decl.declarations.iter().find_map(|declarator| {
                        if let BindingPatternKind::BindingIdentifier(binding_id) =
                            &declarator.id.kind
                            && Self::starts_with_ascii_upper(&binding_id.name)
                            && self.can_be_react_function_component(declarator.init.as_ref())
                            && !Self::is_exported(ctx, node.id())
                        {
                            return Some(binding_id.span);
                        }
                        None
                    })
                }
                AstKind::Function(func) => func.id.as_ref().and_then(|id| {
                    if Self::starts_with_ascii_upper(&id.name) && !Self::is_exported(ctx, node.id())
                    {
                        Some(id.span)
                    } else {
                        None
                    }
                }),
                _ => None,
            })
            .collect()
    }

    fn report_diagnostics(
        ctx: &LintContext,
        export_info: &ExportAnalysis,
        local_components: &[Span],
    ) {
        match (export_info.has_exports, export_info.has_react_export, local_components.is_empty()) {
            (true, true, _) => {
                export_info
                    .non_component_exports
                    .iter()
                    .for_each(|&span| ctx.diagnostic(named_export_components_diagnostic(span)));
                export_info
                    .react_context_exports
                    .iter()
                    .for_each(|&span| ctx.diagnostic(react_context_diagnostic(span)));
            }
            (true, false, false) => {
                for &span in local_components {
                    ctx.diagnostic(local_components_diagnostic(span));
                }
            }
            (false, _, false) => {
                for &span in local_components {
                    ctx.diagnostic(no_export_diagnostic(span));
                }
            }
            _ => {}
        }
    }

    fn analyze_export_default(&self, export_default: &ExportDefaultDeclaration) -> ExportAnalysis {
        let mut analysis = ExportAnalysis::default();

        match &export_default.declaration {
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => return analysis,
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                if let Some(id) = func.id.as_ref() {
                    let export_type = self.classify_export(id.name.as_str(), id.span, true, None);
                    analysis.add_export(export_type);
                } else {
                    analysis.anonymous_span = Some(func.span);
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                if let Some(id) = class.id.as_ref() {
                    let export_type = self.classify_export(id.name.as_str(), id.span, false, None);
                    analysis.add_export(export_type);
                } else {
                    analysis.anonymous_span = Some(class.span);
                }
            }
            ExportDefaultDeclarationKind::CallExpression(call_expr) => {
                if self.is_hoc_call_expression(call_expr) {
                    analysis.has_react_export = true;
                } else {
                    analysis.anonymous_span = Some(export_default.span);
                }
            }
            ExportDefaultDeclarationKind::Identifier(ident) => {
                let export_type =
                    self.classify_export(ident.name.as_str(), ident.span, false, None);
                analysis.add_export(export_type);
            }
            ExportDefaultDeclarationKind::TSAsExpression(ts_as_expr) => {
                return self
                    .analyze_export_default_expression(&ts_as_expr.expression, export_default);
            }
            ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies_expr) => {
                return self.analyze_export_default_expression(
                    &ts_satisfies_expr.expression,
                    export_default,
                );
            }
            _ => {
                analysis.anonymous_span = Some(export_default.span);
            }
        }

        analysis
    }

    fn analyze_export_default_expression(
        &self,
        expr: &Expression,
        export_default: &ExportDefaultDeclaration,
    ) -> ExportAnalysis {
        let mut analysis = ExportAnalysis::default();

        match expr {
            Expression::CallExpression(call_expr) => {
                if self.is_hoc_call_expression(call_expr) {
                    analysis.has_react_export = true;
                } else {
                    analysis.anonymous_span = Some(export_default.span);
                }
            }
            Expression::Identifier(ident) => {
                let export_type =
                    self.classify_export(ident.name.as_str(), ident.span, false, None);
                analysis.add_export(export_type);
            }
            _ => {
                analysis.anonymous_span = Some(export_default.span);
            }
        }

        analysis
    }

    fn analyze_export_named(
        &self,
        ctx: &LintContext,
        export_named: &ExportNamedDeclaration,
    ) -> ExportAnalysis {
        let mut analysis = ExportAnalysis::default();
        if let Some(declaration) = &export_named.declaration {
            let exports = match declaration {
                Declaration::VariableDeclaration(var_decl) => var_decl
                    .declarations
                    .iter()
                    .map(|declarator| match &declarator.id.kind {
                        BindingPatternKind::BindingIdentifier(binding_id) => {
                            let is_func =
                                self.can_be_react_function_component(declarator.init.as_ref());
                            self.classify_export(
                                binding_id.name.as_str(),
                                binding_id.span,
                                is_func,
                                declarator.init.as_ref(),
                            )
                        }
                        _ => ExportType::NonComponent(declarator.id.span()),
                    })
                    .collect::<Vec<_>>(),
                Declaration::FunctionDeclaration(func) => func.id.as_ref().map_or_else(
                    || {
                        ctx.diagnostic(anonymous_components_diagnostic(func.span));
                        vec![]
                    },
                    |id| vec![self.classify_export(id.name.as_str(), id.span, true, None)],
                ),
                Declaration::ClassDeclaration(class) => class.id.as_ref().map_or(vec![], |id| {
                    vec![self.classify_export(id.name.as_str(), id.span, false, None)]
                }),
                Declaration::TSEnumDeclaration(ts_enum) => {
                    vec![ExportType::NonComponent(ts_enum.id.span)]
                }
                _ => vec![],
            };

            for export in exports {
                analysis.add_export(export);
            }
        }

        let specifier_exports: Vec<ExportType> = export_named
            .specifiers
            .iter()
            .map(|export_spec| {
                let exported_name = match &export_spec.exported {
                    ModuleExportName::IdentifierName(name) => Some(name.name.as_str()),
                    ModuleExportName::IdentifierReference(ident) => Some(ident.name.as_str()),
                    ModuleExportName::StringLiteral(_) => None,
                };

                let local_name = export_spec.local.name();
                let span = export_spec.local.span();

                if exported_name == Some("default") {
                    self.classify_export(local_name.as_str(), span, false, None)
                } else if let Some(name) = exported_name {
                    self.classify_export(name, span, false, None)
                } else {
                    ExportType::NonComponent(span)
                }
            })
            .collect();

        for export in specifier_exports {
            analysis.add_export(export);
        }

        analysis
    }

    fn classify_export(
        &self,
        name: &str,
        span: Span,
        is_function: bool,
        init: Option<&Expression>,
    ) -> ExportType {
        if self.allow_export_names.contains(name) {
            return ExportType::Allowed;
        }

        if self.allow_constant_export
            && let Some(init_expr) = init
        {
            let expr_without_ts = Self::skip_ts_expression(init_expr);
            let expr_type = Self::get_expression_type(expr_without_ts);
            if CONSTANT_EXPORT_EXPRESSIONS.contains(expr_type) {
                return ExportType::Allowed;
            }
        }

        if is_function {
            return if Self::starts_with_ascii_upper(name) {
                ExportType::ReactComponent
            } else {
                ExportType::NonComponent(span)
            };
        }

        if let Some(init_expr) = init {
            if let Expression::CallExpression(call_expr) = Self::skip_ts_expression(init_expr) {
                let is_create_context = match &call_expr.callee {
                    Expression::Identifier(ident) => ident.name == "createContext",
                    Expression::StaticMemberExpression(member) => {
                        member.property.name == "createContext"
                    }
                    _ => false,
                };
                if is_create_context {
                    return ExportType::ReactContext(span);
                }
            }

            let expr_without_ts = Self::skip_ts_expression(init_expr);
            let expr_type = Self::get_expression_type(expr_without_ts);
            if NOT_REACT_COMPONENT_EXPRESSION.contains(expr_type) {
                return ExportType::NonComponent(span);
            }
        }

        if Self::starts_with_ascii_upper(name) {
            ExportType::ReactComponent
        } else {
            ExportType::NonComponent(span)
        }
    }

    fn get_expression_type(expr: &Expression<'_>) -> &'static str {
        match expr {
            Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_) => "Literal",
            Expression::UnaryExpression(_) => "UnaryExpression",
            Expression::TemplateLiteral(_) => "TemplateLiteral",
            Expression::BinaryExpression(_) => "BinaryExpression",
            Expression::ArrayExpression(_) => "ArrayExpression",
            Expression::AwaitExpression(_) => "AwaitExpression",
            Expression::ChainExpression(_) => "ChainExpression",
            Expression::ConditionalExpression(_) => "ConditionalExpression",
            Expression::LogicalExpression(_) => "LogicalExpression",
            Expression::ObjectExpression(_) => "ObjectExpression",
            Expression::ThisExpression(_) => "ThisExpression",
            Expression::UpdateExpression(_) => "UpdateExpression",
            _ => "",
        }
    }

    fn is_hoc_call_expression(&self, call_expr: &CallExpression) -> bool {
        let is_callee_hoc = match &call_expr.callee {
            Expression::CallExpression(inner_call) => {
                if let Expression::Identifier(ident) = &inner_call.callee {
                    ident.name == "connect"
                } else {
                    false
                }
            }
            Expression::StaticMemberExpression(member) => {
                if let Expression::Identifier(_) = &member.object {
                    self.is_react_hoc(&member.property.name)
                } else {
                    false
                }
            }
            Expression::Identifier(ident) => self.is_react_hoc(&ident.name),
            _ => false,
        };

        if !is_callee_hoc {
            return false;
        }

        if call_expr.arguments.is_empty() {
            return false;
        }

        call_expr.arguments.first().and_then(|arg| arg.as_expression()).is_some_and(|expr| {
            let expr_without_ts = Self::skip_ts_expression(expr);
            match expr_without_ts {
                Expression::Identifier(_) => true,
                Expression::FunctionExpression(func) => func.id.is_some(),
                Expression::CallExpression(inner_call) => self.is_hoc_call_expression(inner_call),
                _ => false,
            }
        })
    }
}

const CONSTANT_EXPORT_EXPRESSIONS: phf::Set<&'static str> =
    phf::phf_set!["Literal", "UnaryExpression", "TemplateLiteral", "BinaryExpression"];

const NOT_REACT_COMPONENT_EXPRESSION: phf::Set<&'static str> = phf::phf_set![
    "ArrayExpression",
    "AwaitExpression",
    "BinaryExpression",
    "ChainExpression",
    "ConditionalExpression",
    "Literal",
    "LogicalExpression",
    "ObjectExpression",
    "TemplateLiteral",
    "ThisExpression",
    "UnaryExpression",
    "UpdateExpression"
];

#[derive(Debug, Default, Clone)]
struct ExportAnalysis {
    has_exports: bool,
    has_react_export: bool,
    non_component_exports: Vec<Span>,
    react_context_exports: Vec<Span>,
    anonymous_span: Option<Span>,
}

impl ExportAnalysis {
    fn merge(&mut self, other: ExportAnalysis) {
        self.has_exports |= other.has_exports;
        self.has_react_export |= other.has_react_export;
        self.non_component_exports.extend(other.non_component_exports);
        self.react_context_exports.extend(other.react_context_exports);
        if other.anonymous_span.is_some() {
            self.anonymous_span = other.anonymous_span;
        }
    }

    fn add_export(&mut self, export_type: ExportType) {
        match export_type {
            ExportType::ReactComponent => self.has_react_export = true,
            ExportType::NonComponent(span) => self.non_component_exports.push(span),
            ExportType::ReactContext(span) => self.react_context_exports.push(span),
            ExportType::Allowed => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ExportType {
    ReactComponent,
    NonComponent(Span),
    ReactContext(Span),
    Allowed,
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("export function Foo() {};", None),
        ("function Foo() {}; export { Foo };", None),
        ("function Foo() {}; export default Foo;", None),
        ("export default function Foo() {}", None),
        ("export const Foo = () => {};", None),
        ("export const Foo2 = () => {};", None),
        ("export function CMS() {};", None),
        ("export const SVG = forwardRef(() => <svg/>);", None),
        ("export const CMS = () => {};", None),
        ("const Foo = () => {}; export { Foo };", None),
        ("const Foo = () => {}; export default Foo;", None),
        ("const foo = 4; export const Bar = () => {}; export const Baz = () => {};", None),
        ("const foo = () => {}; export const Bar = () => {}; export const Baz = () => {};", None),
        ("export const Foo = () => {}; export const Bar = styled.div`padding-bottom: 6px;`;", None),
        ("export const foo = 3;", None),
        ("const foo = 3; const bar = 'Hello'; export { foo, bar };", None),
        ("export const foo = () => {};", None),
        ("export default function foo () {};", None),
        ("export default memo(function Foo () {});", None),
        ("export default React.memo(function Foo () {});", None),
        ("const Foo = () => {}; export default memo(Foo);", None),
        ("const Foo = () => {}; export default React.memo(Foo);", None),
        ("function Foo() {}; export default memo(Foo);", None),
        ("function Foo() {}; export default React.memo(Foo);", None),
        ("function Foo() {}; export default React.memo(Foo) as typeof Foo;", None),
        ("export type * from './module';", None),
        ("type foo = string; export const Foo = () => null; export type { foo };", None),
        ("export type foo = string; export const Foo = () => null;", None),
        // ("export const foo = () => {}; export const Bar = () => {};", None),
        // (
        //     "export const foo = () => {}; export const Bar = () => {};",
        //     Some(serde_json::json!([{ "checkJS": true }])),
        // ),
        (
            "export const foo = 4; export const Bar = () => {};",
            Some(serde_json::json!([{ "allowConstantExport": true }])),
        ),
        (
            "export const foo = -4; export const Bar = () => {};",
            Some(serde_json::json!([{ "allowConstantExport": true }])),
        ),
        (
            "export const CONSTANT = 'Hello world'; export const Foo = () => {};",
            Some(serde_json::json!([{ "allowConstantExport": true }])),
        ),
        (
            "const foo = 'world'; export const CONSTANT = `Hello ${foo}`; export const Foo = () => {};",
            Some(serde_json::json!([{ "allowConstantExport": true }])),
        ),
        (
            "export const loader = () => {}; export const Bar = () => {};",
            Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }])),
        ),
        (
            "export function loader() {}; export const Bar = () => {};",
            Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }])),
        ),
        (
            "export const loader = () => {}; export const meta = { title: 'Home' };",
            Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }])),
        ),
        ("export { App as default }; const App = () => <>Test</>;", None),
        ("const MyComponent = () => {}; export default connect(() => ({}))(MyComponent);", None),
        ("export const MyComponent = () => {}; export const ChatContext = () => {};", None),
        ("export const MyComponent = () => {}; const MyContext = createContext('test');", None),
        ("export const MyContext = createContext('test');", None),
        (
            "const MyComponent = () => {}; export default observer(MyComponent);",
            Some(serde_json::json!([{ "customHOCs": ["observer"] }])),
        ),
        ("const SomeConstant = 42; export function someUtility() { return SomeConstant }", None),
        (
            "export const MyComponent = () => {}; export const MENU_WIDTH = 232 as const;",
            Some(serde_json::json!([{ "allowConstantExport": true }])),
        ),
        ("export const MyComponent = () => {}; export default memo(MyComponent as any);", None),
        ("export const MyComponent = () => {}; export default memo(MyComponent) as any;", None),
        (
            "export const MyComponent = () => {}; export default memo(forwardRef(MyComponent));",
            None,
        ),
    ];

    let fail = vec![
        ("export const foo = () => {}; export const Bar = () => {};", None),
        (
            "export const foo = () => {}; export const Bar = () => {};",
            Some(serde_json::json!([{ "allowConstantExport": true }])),
        ),
        ("export const foo = 4; export const Bar = () => {};", None),
        ("export function Component() {}; export const Aa = 'a'", None),
        ("const foo = 4; const Bar = () => {}; export { foo, Bar };", None),
        ("export * from './foo';", None),
        ("export default () => {};", None),
        ("export default memo(() => {});", None),
        ("export default function () {};", None),
        ("export const CONSTANT = 3; export const Foo = () => {};", None),
        ("export enum Tab { Home, Settings }; export const Bar = () => {};", None),
        ("const Tab = () => {}; export const tabs = [<Tab />, <Tab />];", None),
        (
            "const App = () => {}; createRoot(document.getElementById('root')).render(<App />);",
            None,
        ),
        (
            r"
        import React from 'react';
        export const CONSTANT = 3; export const Foo = () => {};
        ",
            Some(serde_json::json!([{ "checkJS": true }])),
        ),
        ("export default compose()(MainView);", None),
        (
            "export const loader = () => {}; export const Bar = () => {}; export const foo = () => {};",
            Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }])),
        ),
        (r#"const Foo = () => {}; export { Foo as "🍌"}"#, None),
        (
            "export const MyComponent = () => {}; export const MyContext = createContext('test');",
            None,
        ),
        (
            "export const MyComponent = () => {}; export const MyContext = React.createContext('test');",
            None,
        ),
        ("const MyComponent = () => {}; export default observer(MyComponent);", None),
    ];

    Tester::new(OnlyExportComponents::NAME, OnlyExportComponents::PLUGIN, pass, fail)
        .test_and_snapshot();
}
