use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, Expression, MetaProperty, NewExpression,
        VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

const PATH_MODULES: [&str; 2] = ["path", "node:path"];
const URL_MODULES: [&str; 2] = ["url", "node:url"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum CheckKind {
    Module,
    Property,
}

#[derive(Clone, Copy)]
enum ProblemKind {
    Dirname,
    Filename,
}

impl ProblemKind {
    fn message(self) -> &'static str {
        match self {
            Self::Dirname => "Do not construct dirname.",
            Self::Filename => "Do not construct filename using `fileURLToPath()`.",
        }
    }

    fn property(self) -> &'static str {
        match self {
            Self::Dirname => "dirname",
            Self::Filename => "filename",
        }
    }
}

fn prefer_import_meta_diagnostic(span: Span, kind: ProblemKind) -> OxcDiagnostic {
    OxcDiagnostic::warn(kind.message())
        .with_help(format!("Replace this expression with `import.meta.{}`.", kind.property()))
        .with_label(span)
}

fn report_problem(ctx: &LintContext<'_>, span: Span, kind: ProblemKind) {
    ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(span, kind), |fixer| {
        fixer.replace(span, format!("import.meta.{}", kind.property()))
    });
}

#[derive(Debug, Default, Clone)]
pub struct PreferImportMetaProperties;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `import.meta.{dirname,filename}` over legacy
    /// techniques for getting file paths.
    ///
    /// ### Why is this bad?
    ///
    /// Starting with Node.js 20.11, `import.meta.dirname` and `import.meta.filename`
    /// have been introduced in ES modules.
    /// `import.meta.filename` is equivalent to `url.fileURLToPath(import.meta.url)`.
    /// `import.meta.dirname` is equivalent to `path.dirname(import.meta.filename)`.
    /// This rule replaces legacy patterns with `import.meta.dirname` and `import.meta.filename`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import path from "node:path"
    /// import { fileURLToPath } from "url";
    ///
    /// const filename = fileURLToPath(import.meta.url);
    /// const dirname = path.dirname(fileURLToPath(import.meta.url));
    /// const dirname = path.dirname(import.meta.filename)
    /// const dirname = fileURLToPath(new URL('.', import.meta.url))
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const filename = import.meta.filename;
    /// const dirname = import.meta.dirname;
    /// ```
    PreferImportMetaProperties,
    unicorn,
    pedantic,
    fix,
    version = "1.59.0",
);

impl Rule for PreferImportMetaProperties {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MetaProperty(meta_property) = node.kind() else { return };
        if !is_import_meta(meta_property) {
            return;
        }

        let member_expression_id = ctx.nodes().parent_id(meta_property.node_id());
        let AstKind::StaticMemberExpression(member_expression) =
            ctx.nodes().kind(member_expression_id)
        else {
            return;
        };
        if member_expression.optional {
            return;
        }

        match member_expression.property.name.as_str() {
            "url" => {
                let parent_id = ctx.nodes().parent_id(member_expression_id);
                let member_expression_span = ctx.nodes().kind(member_expression_id).span();

                if let AstKind::CallExpression(parent) = ctx.nodes().kind(parent_id)
                    && is_url_file_url_to_path_call(parent, ctx)
                    && has_argument_expression(&parent.arguments, 0, member_expression_span)
                {
                    iterate_problems_from_filename(parent.node_id(), true, ctx);
                    return;
                }

                if let AstKind::NewExpression(new_url) = ctx.nodes().kind(parent_id)
                    && is_url_constructor(new_url, ctx)
                {
                    let url_parent_id = ctx.nodes().parent_id(new_url.node_id());
                    let AstKind::CallExpression(url_parent) = ctx.nodes().kind(url_parent_id)
                    else {
                        return;
                    };

                    if !is_url_file_url_to_path_call(url_parent, ctx)
                        || !has_argument_expression(&url_parent.arguments, 0, new_url.span)
                    {
                        return;
                    }

                    if new_url.arguments.len() == 1
                        && has_argument_expression(&new_url.arguments, 0, member_expression_span)
                    {
                        iterate_problems_from_filename(url_parent.node_id(), true, ctx);
                        return;
                    }

                    if new_url.arguments.len() == 2
                        && is_parent_literal(&new_url.arguments[0])
                        && has_argument_expression(&new_url.arguments, 1, member_expression_span)
                    {
                        report_problem(ctx, url_parent.span, ProblemKind::Dirname);
                    }
                }
            }
            "filename" => iterate_problems_from_filename(member_expression.node_id(), false, ctx),
            _ => {}
        }
    }
}

/// Returns `true` if `call` is `process.getBuiltinModule("one_of_modules")`.
fn is_process_get_builtin_module_call(call: &CallExpression<'_>, modules: &[&str]) -> bool {
    if call.optional || call.arguments.len() != 1 || call.arguments[0].is_spread() {
        return false;
    }
    let Expression::StaticMemberExpression(callee_member) = &call.callee else { return false };
    if callee_member.optional {
        return false;
    }
    let Expression::Identifier(process_ident) = &callee_member.object else { return false };
    if process_ident.name != "process" || callee_member.property.name != "getBuiltinModule" {
        return false;
    }
    matches!(
        &call.arguments[0],
        Argument::StringLiteral(lit) if modules.contains(&lit.value.as_str())
    )
}

fn is_import_meta(meta_property: &MetaProperty<'_>) -> bool {
    meta_property.meta.name == "import" && meta_property.property.name == "meta"
}

fn is_parent_literal(argument: &Argument<'_>) -> bool {
    matches!(
        argument,
        Argument::StringLiteral(lit) if lit.value.as_str() == "." || lit.value.as_str() == "./"
    )
}

fn has_argument_expression(arguments: &[Argument<'_>], index: usize, node_span: Span) -> bool {
    arguments
        .get(index)
        .and_then(Argument::as_expression)
        .is_some_and(|expr| expr.span() == node_span)
}

fn is_url_constructor(new_expression: &NewExpression<'_>, ctx: &LintContext<'_>) -> bool {
    if !(1..=2).contains(&new_expression.arguments.len()) {
        return false;
    }

    let Expression::Identifier(identifier) = &new_expression.callee else { return false };
    if identifier.name != "URL" {
        return false;
    }

    ctx.scoping().get_reference(identifier.reference_id()).symbol_id().is_none()
}

fn is_url_file_url_to_path_call(
    call_expression: &CallExpression<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    is_node_builtin_module_function_call(call_expression, &URL_MODULES, "fileURLToPath", ctx)
}

fn is_path_dirname_call(call_expression: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    is_node_builtin_module_function_call(call_expression, &PATH_MODULES, "dirname", ctx)
}

fn is_node_builtin_module_function_call(
    node: &CallExpression<'_>,
    modules: &[&str],
    function_name: &str,
    ctx: &LintContext<'_>,
) -> bool {
    if node.optional || node.arguments.len() != 1 || node.arguments[0].is_spread() {
        return false;
    }
    let mut visited = FxHashSet::default();
    check_expression(&node.callee, CheckKind::Property, modules, function_name, ctx, &mut visited)
}

fn check_expression(
    node: &Expression<'_>,
    check_kind: CheckKind,
    modules: &[&str],
    function_name: &str,
    ctx: &LintContext<'_>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    match node {
        Expression::StaticMemberExpression(member_expression) => {
            if !matches!(check_kind, CheckKind::Property)
                || member_expression.optional
                || member_expression.property.name != function_name
            {
                return false;
            }

            check_expression(
                &member_expression.object,
                CheckKind::Module,
                modules,
                function_name,
                ctx,
                visited,
            )
        }
        Expression::CallExpression(call_expression) => {
            matches!(check_kind, CheckKind::Module)
                && is_process_get_builtin_module_call(call_expression, modules)
        }
        Expression::Identifier(identifier) => {
            let reference = ctx.scoping().get_reference(identifier.reference_id());
            let Some(symbol_id) = reference.symbol_id() else { return false };
            if !visited.insert(symbol_id) {
                return false;
            }
            check_definition(symbol_id, check_kind, modules, function_name, ctx, visited)
        }
        _ => false,
    }
}

fn check_definition(
    symbol_id: SymbolId,
    check_kind: CheckKind,
    modules: &[&str],
    function_name: &str,
    ctx: &LintContext<'_>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
    match ctx.nodes().kind(declaration_id) {
        AstKind::ImportSpecifier(import_specifier) => {
            import_declaration_source(declaration_id, ctx).is_some_and(|source| {
                matches!(check_kind, CheckKind::Property)
                    && modules.contains(&source)
                    && import_specifier.imported.name() == function_name
            })
        }
        AstKind::ImportDefaultSpecifier(_) | AstKind::ImportNamespaceSpecifier(_) => {
            matches!(check_kind, CheckKind::Module)
                && import_declaration_source(declaration_id, ctx)
                    .is_some_and(|source| modules.contains(&source))
        }
        AstKind::VariableDeclarator(variable_declarator) => check_variable_declarator(
            variable_declarator,
            symbol_id,
            check_kind,
            modules,
            function_name,
            ctx,
            visited,
        ),
        _ => false,
    }
}

fn import_declaration_source<'a>(
    declaration_id: NodeId,
    ctx: &'a LintContext<'_>,
) -> Option<&'a str> {
    let AstKind::ImportDeclaration(import_declaration) = ctx.nodes().parent_kind(declaration_id)
    else {
        return None;
    };
    Some(import_declaration.source.value.as_str())
}

fn check_variable_declarator(
    variable_declarator: &VariableDeclarator<'_>,
    symbol_id: SymbolId,
    check_kind: CheckKind,
    modules: &[&str],
    function_name: &str,
    ctx: &LintContext<'_>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    if !variable_declarator.kind.is_const() {
        return false;
    }

    let Some(init) = &variable_declarator.init else { return false };

    match &variable_declarator.id {
        BindingPattern::BindingIdentifier(binding_identifier)
            if binding_identifier.symbol_id() == symbol_id =>
        {
            check_expression(init, check_kind, modules, function_name, ctx, visited)
        }
        BindingPattern::ObjectPattern(object_pattern) => {
            for property in &object_pattern.properties {
                if property.computed {
                    continue;
                }
                let BindingPattern::BindingIdentifier(binding_identifier) = &property.value else {
                    continue;
                };
                if binding_identifier.symbol_id() != symbol_id {
                    continue;
                }
                let Some(property_name) = property.key.static_name() else { return false };
                if !matches!(check_kind, CheckKind::Property) || property_name != function_name {
                    return false;
                }
                return check_expression(
                    init,
                    CheckKind::Module,
                    modules,
                    function_name,
                    ctx,
                    visited,
                );
            }
            false
        }
        _ => false,
    }
}

fn iterate_problems_from_filename(
    node_id: NodeId,
    report_filename_node: bool,
    ctx: &LintContext<'_>,
) {
    let parent_id = ctx.nodes().parent_id(node_id);
    let node_span = ctx.nodes().kind(node_id).span();

    if let AstKind::CallExpression(parent) = ctx.nodes().kind(parent_id)
        && is_path_dirname_call(parent, ctx)
        && has_argument_expression(&parent.arguments, 0, node_span)
    {
        report_problem(ctx, parent.span, ProblemKind::Dirname);
        return;
    }

    if report_filename_node {
        report_problem(ctx, ctx.nodes().kind(node_id).span(), ProblemKind::Filename);
    }

    let AstKind::VariableDeclarator(parent) = ctx.nodes().kind(parent_id) else { return };
    if !parent.kind.is_const() || parent.init.as_ref().is_none_or(|init| init.span() != node_span) {
        return;
    }

    let BindingPattern::BindingIdentifier(binding_identifier) = &parent.id else { return };
    for reference in
        ctx.semantic().symbol_references(binding_identifier.symbol_id()).filter(|r| r.is_read())
    {
        let reference_parent_id = ctx.nodes().parent_id(reference.node_id());
        let AstKind::CallExpression(parent) = ctx.nodes().kind(reference_parent_id) else {
            continue;
        };
        if is_path_dirname_call(parent, ctx)
            && has_argument_expression(
                &parent.arguments,
                0,
                ctx.nodes().get_node(reference.node_id()).kind().span(),
            )
        {
            report_problem(ctx, parent.span, ProblemKind::Dirname);
        }
    }
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};

    let pass = vec![
        "const __dirname = import.meta.dirname;",
        "const __filename = import.meta.filename;",
        r#"import path from "path";
            const dirUrl = path.dirname(import.meta.url);"#,
        "const url = import.meta.url;",
        r#"const dirname = new URL(".", import.meta.url).pathname;"#,
        "const filename = new URL(import.meta.url).pathname;",
        "const filename = fileURLToPath(import.meta.url);",
        "const dirname = path.dirname(import.meta.filename);",
        r#"import path from "path";
            // It is the same as dirname on macOS but returns different results on Windows.
            const notDirname = path.dirname(new URL(import.meta.url).pathname);"#,
        "// path is not initialized
            let path;
            const dirname = path.dirname(import.meta.filename);",
        r#"// path is unknown property
            const { path } = process.getBuiltinModule("node:path");
            const dirname = path.dirname(import.meta.filename);"#,
        r#"const { dirname } = process.getBuiltinModule("node:path");
            // dirname()() is unknown
            const x = dirname(x)(import.meta.filename);"#,
        "// path is unknown
            const path = new X();
            const dirname = path.dirname(import.meta.filename);",
        "// path is unknown
            const path = path;
            const dirname = path.dirname(import.meta.filename);",
        r#"// path is unknown
            const [path] = process.getBuiltinModule("node:path");
            const dirname = path.dirname(import.meta.filename);"#,
        r#"import path from "path";
            const dirname = path?.dirname(import.meta.filename);"#,
        r#"import path from "path";
            const dirname = path[dirname](import.meta.filename);"#,
        r#"import path from "path";
            const dirname = path["dirname"](import.meta.filename);"#,
        r#"import path from "path";
            const dirname = path.dirname?.(import.meta.filename);"#,
        r#"const { [fileURLToPath]: fileURLToPath } = process.getBuiltinModule("node:url");
            const filename = fileURLToPath(import.meta.url);"#,
        r#"const { ["fileURLToPath"]: fileURLToPath } = process.getBuiltinModule("node:url");
            const filename = fileURLToPath(import.meta.url);"#,
        r#"import {fileURLToPath} from "node:url";
            class Foo {
                constructor() {
                    const filename = fileURLToPath(new.target.url)
                }
            }"#,
        r#"import {fileURLToPath} from "node:url";
            const filename = fileURLToPath(import.meta?.url)"#,
        r#"import {fileURLToPath} from "node:url";
            const filename = fileURLToPath(import.meta['url'])"#,
        // URL is shadowed by a local binding — should not flag
        r#"import { fileURLToPath } from "url";
            const URL = class {};
            const filename = fileURLToPath(new URL(import.meta.url));"#,
        // new URL with non-import.meta.url argument — should not flag
        r#"import { fileURLToPath } from "url";
            const filename = fileURLToPath(new URL("https://example.com"));"#,
        r#"import { fileURLToPath } from "url";
            const dirname = fileURLToPath(new URL(".", "https://example.com"));"#,
    ];

    let fail = vec![
        r#"import path from "path";
            import { fileURLToPath } from "url";
            const dirname = path.dirname(fileURLToPath(import.meta.url));"#,
        r#"import path from "path";
            const dirname = path.dirname(import.meta.filename);"#,
        r#"import { fileURLToPath } from "url";
            const dirname = fileURLToPath(new URL(".", import.meta.url));"#,
        r#"import { fileURLToPath } from "url";
            const dirname = fileURLToPath(new URL("./", import.meta.url));"#,
        r#"import { fileURLToPath } from "url";
            const filename = fileURLToPath(import.meta.url);"#,
        r#"import { fileURLToPath } from "url";
            const filename = fileURLToPath(new URL(import.meta.url));"#,
        r#"import path from "node:path";
            import { fileURLToPath } from "node:url";
            const dirname = path.dirname(fileURLToPath(import.meta.url));"#,
        r#"import { fileURLToPath } from "node:url";
            const filename = fileURLToPath(import.meta.url);"#,
        r#"import * as path from "node:path";
            import url from "node:url";
            const dirname = path.dirname(url.fileURLToPath(import.meta.url));"#,
        r#"import url from "node:url";
            const filename = url.fileURLToPath(import.meta.url);"#,
        r#"import path from "node:path";
            import { fileURLToPath } from "node:url";
            const __filename = fileURLToPath(import.meta.url);
            const __dirname = path.dirname(__filename);"#,
        r#"import path from "node:path";
            const __filename = import.meta.filename;
            const __dirname = path.dirname(__filename);"#,
        r#"const path = process.getBuiltinModule("node:path");
            const { fileURLToPath } = process.getBuiltinModule("node:url");
            const filename = fileURLToPath(import.meta.url);
            const dirname = path.dirname(filename);"#,
        r#"const { fileURLToPath: renamed } = process.getBuiltinModule("node:url");
            const filename = renamed(import.meta.url);"#,
        r#"import { fileURLToPath as renamed } from "node:url";
            const filename = renamed(import.meta.url);"#,
        r#"const path = process.getBuiltinModule("path");
            const { fileURLToPath } = process.getBuiltinModule("url");
            const filename = fileURLToPath(import.meta.url);
            const dirname = path.dirname(filename);"#,
        r#"const filename = process.getBuiltinModule("node:url").fileURLToPath(import.meta.url);
            const dirname = process.getBuiltinModule("node:path").dirname(filename);"#,
        // inline-chained without subsequent dirname — should emit filename diagnostic
        r#"const filename = process.getBuiltinModule("node:url").fileURLToPath(import.meta.url);"#,
    ];

    let fix: Vec<ExpectFixTestCase> = vec![
        // filename: fileURLToPath(import.meta.url) → import.meta.filename
        (
            r#"import { fileURLToPath } from "url";
            const filename = fileURLToPath(import.meta.url);"#,
            r#"import { fileURLToPath } from "url";
            const filename = import.meta.filename;"#,
        )
            .into(),
        // dirname: path.dirname(fileURLToPath(import.meta.url)) → import.meta.dirname
        (
            r#"import path from "path";
            import { fileURLToPath } from "url";
            const dirname = path.dirname(fileURLToPath(import.meta.url));"#,
            r#"import path from "path";
            import { fileURLToPath } from "url";
            const dirname = import.meta.dirname;"#,
        )
            .into(),
        // dirname via new URL(".", …)
        (
            r#"import { fileURLToPath } from "url";
            const dirname = fileURLToPath(new URL(".", import.meta.url));"#,
            r#"import { fileURLToPath } from "url";
            const dirname = import.meta.dirname;"#,
        )
            .into(),
        // dirname via import.meta.filename passed to path.dirname
        (
            r#"import path from "path";
            const dirname = path.dirname(import.meta.filename);"#,
            r#"import path from "path";
            const dirname = import.meta.dirname;"#,
        )
            .into(),
        // dirname via multi-step: const __filename = …; path.dirname(__filename)
        (
            r#"import path from "node:path";
            import { fileURLToPath } from "node:url";
            const __filename = fileURLToPath(import.meta.url);
            const __dirname = path.dirname(__filename);"#,
            r#"import path from "node:path";
            import { fileURLToPath } from "node:url";
            const __filename = import.meta.filename;
            const __dirname = import.meta.dirname;"#,
        )
            .into(),
    ];

    Tester::new(PreferImportMetaProperties::NAME, PreferImportMetaProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
