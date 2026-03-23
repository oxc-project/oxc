use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, BindingPattern, CallExpression, Expression,
        ImportDeclarationSpecifier, NewExpression, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::node::NodeId;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

const PATH_MODULES: [&str; 2] = ["path", "node:path"];
const URL_MODULES: [&str; 2] = ["url", "node:url"];

#[derive(Default)]
struct ImportSymbols {
    /// SymbolIds that resolve to `fileURLToPath` from url/node:url
    filename_fn_ids: FxHashSet<SymbolId>,
    /// SymbolIds that resolve to `dirname` from path/node:path
    dirname_fn_ids: FxHashSet<SymbolId>,
    /// SymbolIds that are the url module object (default/namespace import)
    url_mod_ids: FxHashSet<SymbolId>,
    /// SymbolIds that are the path module object (default/namespace import)
    path_mod_ids: FxHashSet<SymbolId>,
    /// True when any `process.getBuiltinModule("node:url/path")` call was seen.
    /// Set even for assigned-to-variable forms, but its primary purpose is to
    /// prevent early-exit when the call is inline-chained (no symbol ID produced).
    has_inline_module_calls: bool,
}

impl ImportSymbols {
    fn is_empty(&self) -> bool {
        self.filename_fn_ids.is_empty()
            && self.dirname_fn_ids.is_empty()
            && self.url_mod_ids.is_empty()
            && self.path_mod_ids.is_empty()
    }

    fn has_inline_module_calls(&self) -> bool {
        self.has_inline_module_calls
    }
}

fn insert_module_obj(symbol_id: SymbolId, is_url: bool, sym: &mut ImportSymbols) {
    if is_url {
        sym.url_mod_ids.insert(symbol_id);
    } else {
        sym.path_mod_ids.insert(symbol_id);
    }
}

fn collect_relevant_symbols(ctx: &LintContext<'_>) -> ImportSymbols {
    let mut sym = ImportSymbols::default();

    for node in ctx.nodes().iter() {
        match node.kind() {
            AstKind::ImportDeclaration(decl) => {
                let source = decl.source.value.as_str();
                let is_url = URL_MODULES.contains(&source);
                let is_path = PATH_MODULES.contains(&source);
                if !is_url && !is_path {
                    continue;
                }
                let Some(specifiers) = &decl.specifiers else { continue };
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(s) => {
                            let imported = s.imported.name().as_str();
                            let symbol_id = s.local.symbol_id();
                            if is_url && imported == "fileURLToPath" {
                                sym.filename_fn_ids.insert(symbol_id);
                            } else if is_path && imported == "dirname" {
                                sym.dirname_fn_ids.insert(symbol_id);
                            }
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                            insert_module_obj(s.local.symbol_id(), is_url, &mut sym);
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                            insert_module_obj(s.local.symbol_id(), is_url, &mut sym);
                        }
                    }
                }
            }
            // Handles: const { fileURLToPath } = process.getBuiltinModule("node:url")
            // and: const path = process.getBuiltinModule("node:path")
            AstKind::VariableDeclarator(decl) => {
                collect_get_builtin_symbols(decl, &mut sym);
            }
            // Detects any process.getBuiltinModule("node:url/path") call.
            // Primarily catches the inline-chained form (e.g.
            //   process.getBuiltinModule("node:url").fileURLToPath(import.meta.url))
            // where no symbol ID is produced; also fires for assigned-to-variable forms,
            // which is harmless since those already populate the symbol sets.
            AstKind::CallExpression(call) => {
                if !sym.has_inline_module_calls
                    && (is_process_get_builtin_module_call(call, &URL_MODULES)
                        || is_process_get_builtin_module_call(call, &PATH_MODULES))
                {
                    sym.has_inline_module_calls = true;
                }
            }
            _ => {}
        }
    }
    sym
}

/// Handles `const { fileURLToPath } = process.getBuiltinModule("node:url")`
/// and `const path = process.getBuiltinModule("node:path")` forms.
fn collect_get_builtin_symbols(decl: &VariableDeclarator<'_>, sym: &mut ImportSymbols) {
    let Some(init) = &decl.init else { return };
    let Expression::CallExpression(call) = init else { return };

    let is_url = is_process_get_builtin_module_call(call, &URL_MODULES);
    let is_path = is_process_get_builtin_module_call(call, &PATH_MODULES);
    if !is_url && !is_path {
        return;
    }

    match &decl.id {
        // const { fileURLToPath } = process.getBuiltinModule("node:url")
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                if prop.computed {
                    continue;
                }
                let BindingPattern::BindingIdentifier(id) = &prop.value else { continue };
                let Some(key) = prop.key.static_name() else { continue };
                let symbol_id = id.symbol_id();
                if is_url && key == "fileURLToPath" {
                    sym.filename_fn_ids.insert(symbol_id);
                } else if is_path && key == "dirname" {
                    sym.dirname_fn_ids.insert(symbol_id);
                }
            }
        }
        // const path = process.getBuiltinModule("node:path")
        BindingPattern::BindingIdentifier(id) => {
            insert_module_obj(id.symbol_id(), is_url, sym);
        }
        _ => {}
    }
}

fn prefer_import_meta_diagnostic(span: Span, property: &'static str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `import.meta.{property}`."))
        .with_help(format!("Replace this expression with `import.meta.{property}`."))
        .with_label(span)
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
);

impl Rule for PreferImportMetaProperties {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let sym = collect_relevant_symbols(ctx);
        if sym.is_empty() && !sym.has_inline_module_calls() {
            return;
        }
        let filename_val_ids = collect_filename_values(&sym, ctx);
        check_patterns(&sym, &filename_val_ids, ctx);
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

/// Returns true if `call` is a non-optional single-arg (non-spread) call.
fn is_single_arg_call(call: &CallExpression<'_>) -> bool {
    !call.optional && call.arguments.len() == 1 && !call.arguments[0].is_spread()
}

#[derive(Clone, Copy)]
enum ModuleKind {
    Filename,
    Dirname,
}

/// Returns true if `expr` is the module function for `kind`:
/// - `Filename`: `fileURLToPath` identifier or `url.fileURLToPath` member expr
/// - `Dirname`:  `dirname` identifier or `path.dirname` member expr
fn is_callee_module_fn<'a>(
    expr: &Expression<'a>,
    kind: ModuleKind,
    sym: &ImportSymbols,
    ctx: &LintContext<'a>,
) -> bool {
    let (fn_ids, property_name, mod_ids, modules): (_, &str, _, &[&str]) = match kind {
        ModuleKind::Filename => {
            (&sym.filename_fn_ids, "fileURLToPath", &sym.url_mod_ids, URL_MODULES.as_slice())
        }
        ModuleKind::Dirname => {
            (&sym.dirname_fn_ids, "dirname", &sym.path_mod_ids, PATH_MODULES.as_slice())
        }
    };
    match expr {
        Expression::Identifier(ident) => {
            let reference = ctx.scoping().get_reference(ident.reference_id());
            reference.symbol_id().is_some_and(|id| fn_ids.contains(&id))
        }
        Expression::StaticMemberExpression(member_expr)
            if !member_expr.optional && member_expr.property.name == property_name =>
        {
            is_module_obj_expr(&member_expr.object, mod_ids, modules, ctx)
        }
        _ => false,
    }
}

/// Returns true if `expr` is a module object identifier (resolves to a SymbolId in mod_ids),
/// or an inline process.getBuiltinModule(...) call for one of `modules`.
fn is_module_obj_expr<'a>(
    expr: &Expression<'a>,
    mod_ids: &FxHashSet<SymbolId>,
    modules: &[&str],
    ctx: &LintContext<'a>,
) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            let reference = ctx.scoping().get_reference(ident.reference_id());
            reference.symbol_id().is_some_and(|id| mod_ids.contains(&id))
        }
        // process.getBuiltinModule("node:path").dirname — no symbol, inline call
        Expression::CallExpression(call) => is_process_get_builtin_module_call(call, modules),
        _ => false,
    }
}

/// Returns true if `expr` is `import.meta.<property>` (non-optional).
fn is_import_meta_property(expr: &Expression<'_>, property: &str) -> bool {
    let Expression::StaticMemberExpression(outer) = expr else { return false };
    !outer.optional
        && outer.property.name == property
        && matches!(
            &outer.object,
            Expression::MetaProperty(mp)
                if mp.meta.name == "import" && mp.property.name == "meta"
        )
}

// Phase 2: data-flow — collect variables holding filename values

/// Phase 2: collect SymbolIds for variables that hold a filename value.
///
/// Covers:
/// - `const __filename = fileURLToPath(import.meta.url)`
/// - `const __filename = import.meta.filename`
/// - `__filename = fileURLToPath(import.meta.url)`   ← assignment form
/// - `__filename = import.meta.filename`              ← assignment form
fn collect_filename_values(sym: &ImportSymbols, ctx: &LintContext<'_>) -> FxHashSet<SymbolId> {
    let mut ids = FxHashSet::default();

    for node in ctx.nodes().iter() {
        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                let BindingPattern::BindingIdentifier(id) = &decl.id else { continue };
                if let Some(init) = &decl.init
                    && is_filename_value_expr(init, sym, ctx)
                {
                    ids.insert(id.symbol_id());
                }
            }
            AstKind::AssignmentExpression(assign) => {
                let AssignmentTarget::AssignmentTargetIdentifier(target) = &assign.left else {
                    continue;
                };
                let reference = ctx.scoping().get_reference(target.reference_id());
                let Some(symbol_id) = reference.symbol_id() else { continue };
                if is_filename_value_expr(&assign.right, sym, ctx) {
                    ids.insert(symbol_id);
                }
            }
            _ => {}
        }
    }
    ids
}

/// Returns true if `expr` evaluates to a filename string value
/// (i.e., is equivalent to `import.meta.filename`).
fn is_filename_value_expr<'a>(
    expr: &Expression<'a>,
    sym: &ImportSymbols,
    ctx: &LintContext<'a>,
) -> bool {
    // fileURLToPath(import.meta.url)
    if let Expression::CallExpression(call) = expr
        && is_single_arg_call(call)
        && is_callee_module_fn(&call.callee, ModuleKind::Filename, sym, ctx)
    {
        return call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .is_some_and(|e| is_import_meta_property(e, "url"));
    }
    // import.meta.filename
    is_import_meta_property(expr, "filename")
}

/// Phase 3: walk all MetaProperty nodes and report fixable patterns.
fn check_patterns(
    sym: &ImportSymbols,
    filename_val_ids: &FxHashSet<SymbolId>,
    ctx: &LintContext<'_>,
) {
    for node in ctx.nodes().iter() {
        let AstKind::MetaProperty(mp) = node.kind() else { continue };
        if mp.meta.name != "import" || mp.property.name != "meta" {
            continue;
        }

        // Parent must be a non-optional static member expression (import.meta.url/filename/…)
        let parent_id = ctx.nodes().parent_id(node.id());
        let AstKind::StaticMemberExpression(member_expr) = ctx.nodes().kind(parent_id) else {
            continue;
        };
        if member_expr.optional {
            continue;
        }

        match member_expr.property.name.as_str() {
            "url" => handle_import_meta_url(parent_id, sym, ctx),
            "filename" => {
                try_report_dirname(parent_id, sym, ctx);
            }
            _ => {}
        }
    }

    // Also check filename_val_ids: variables assigned filename values,
    // looking for path.dirname(variable) usages.
    for &symbol_id in filename_val_ids {
        for reference in ctx.semantic().symbol_references(symbol_id).filter(|r| r.is_read()) {
            let ref_parent_id = ctx.nodes().parent_id(reference.node_id());
            let AstKind::CallExpression(call) = ctx.nodes().kind(ref_parent_id) else { continue };
            if !is_single_arg_call(call)
                || !is_callee_module_fn(&call.callee, ModuleKind::Dirname, sym, ctx)
            {
                continue;
            }
            ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(call.span, "dirname"), |fixer| {
                fixer.replace(call.span, "import.meta.dirname")
            });
        }
    }
}

fn handle_import_meta_url(url_node_id: NodeId, sym: &ImportSymbols, ctx: &LintContext<'_>) {
    let parent_id = ctx.nodes().parent_id(url_node_id);
    match ctx.nodes().kind(parent_id) {
        AstKind::CallExpression(call)
            if is_single_arg_call(call)
                && is_callee_module_fn(&call.callee, ModuleKind::Filename, sym, ctx) =>
        {
            if !try_report_dirname(parent_id, sym, ctx) {
                ctx.diagnostic_with_fix(
                    prefer_import_meta_diagnostic(call.span, "filename"),
                    |fixer| fixer.replace(call.span, "import.meta.filename"),
                );
            }
        }
        AstKind::NewExpression(new_expr) => {
            handle_new_url(new_expr, parent_id, sym, ctx);
        }
        _ => {}
    }
}

fn handle_new_url(
    new_expr: &NewExpression<'_>,
    new_expr_node_id: NodeId,
    sym: &ImportSymbols,
    ctx: &LintContext<'_>,
) {
    let Expression::Identifier(url_ident) = &new_expr.callee else { return };
    if url_ident.name != "URL" {
        return;
    }
    // Verify URL is the global built-in, not a locally shadowed binding.
    let reference = ctx.scoping().get_reference(url_ident.reference_id());
    if reference.symbol_id().is_some() {
        return;
    }

    let property: &str = match new_expr.arguments.len() {
        1 => "filename",
        2 => {
            let first_is_dot = matches!(
                &new_expr.arguments[0],
                Argument::StringLiteral(lit) if lit.value == "." || lit.value == "./"
            );
            if !first_is_dot {
                return;
            }
            "dirname"
        }
        _ => return,
    };

    let grandparent_id = ctx.nodes().parent_id(new_expr_node_id);
    let AstKind::CallExpression(outer) = ctx.nodes().kind(grandparent_id) else { return };
    if is_single_arg_call(outer)
        && is_callee_module_fn(&outer.callee, ModuleKind::Filename, sym, ctx)
    {
        ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(outer.span, property), |fixer| {
            fixer.replace(outer.span, format!("import.meta.{property}"))
        });
    }
}

/// Checks whether the filename-valued node at `filename_node_id` is used as an argument
/// to `path.dirname(...)`, either directly or via a variable.
///
/// - **Direct case** (`path.dirname(<filename_node>)`): emits the dirname diagnostic and
///   returns `true`.
/// - **Variable case** (`const v = <filename_node>; path.dirname(v)`): returns `true` if
///   any read of `v` appears inside a `path.dirname` call, but does NOT emit — the
///   `filename_val_ids` sweep in `check_patterns` owns that emission.
/// - **Other**: returns `false`.
///
/// The return value is used by callers to decide whether to suppress a filename diagnostic
/// (since a dirname diagnostic will already be produced for the same value).
fn try_report_dirname(
    filename_node_id: NodeId,
    sym: &ImportSymbols,
    ctx: &LintContext<'_>,
) -> bool {
    let parent_id = ctx.nodes().parent_id(filename_node_id);

    match ctx.nodes().kind(parent_id) {
        // Direct: path.dirname(import.meta.filename) or path.dirname(fileURLToPath(…))
        AstKind::CallExpression(call)
            if is_single_arg_call(call)
                && is_callee_module_fn(&call.callee, ModuleKind::Dirname, sym, ctx) =>
        {
            ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(call.span, "dirname"), |fixer| {
                fixer.replace(call.span, "import.meta.dirname")
            });
            true
        }
        // Stored: const __filename = …; path.dirname(__filename)
        AstKind::VariableDeclarator(var_decl) => {
            let BindingPattern::BindingIdentifier(bind) = &var_decl.id else { return false };
            let symbol_id = bind.symbol_id();
            // Check if any read reference is used in a dirname call.
            // Do NOT emit here — the filename_val_ids sweep in check_patterns owns that emission.
            ctx.semantic().symbol_references(symbol_id).filter(|r| r.is_read()).any(|reference| {
                let ref_parent_id = ctx.nodes().parent_id(reference.node_id());
                let AstKind::CallExpression(call) = ctx.nodes().kind(ref_parent_id) else {
                    return false;
                };
                is_single_arg_call(call)
                    && is_callee_module_fn(&call.callee, ModuleKind::Dirname, sym, ctx)
            })
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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
        // assignment form — new detection
        r#"import path from "node:path";
    let __filename;
    __filename = import.meta.filename;
    const __dirname = path.dirname(__filename);"#,
    ];

    let fix = vec![
        // filename: fileURLToPath(import.meta.url) → import.meta.filename
        (
            r#"import { fileURLToPath } from "url";
            const filename = fileURLToPath(import.meta.url);"#,
            r#"import { fileURLToPath } from "url";
            const filename = import.meta.filename;"#,
        ),
        // dirname: path.dirname(fileURLToPath(import.meta.url)) → import.meta.dirname
        (
            r#"import path from "path";
            import { fileURLToPath } from "url";
            const dirname = path.dirname(fileURLToPath(import.meta.url));"#,
            r#"import path from "path";
            import { fileURLToPath } from "url";
            const dirname = import.meta.dirname;"#,
        ),
        // dirname via new URL(".", …)
        (
            r#"import { fileURLToPath } from "url";
            const dirname = fileURLToPath(new URL(".", import.meta.url));"#,
            r#"import { fileURLToPath } from "url";
            const dirname = import.meta.dirname;"#,
        ),
        // dirname via import.meta.filename passed to path.dirname
        (
            r#"import path from "path";
            const dirname = path.dirname(import.meta.filename);"#,
            r#"import path from "path";
            const dirname = import.meta.dirname;"#,
        ),
        // dirname via multi-step: const __filename = …; path.dirname(__filename)
        (
            r#"import path from "node:path";
            import { fileURLToPath } from "node:url";
            const __filename = fileURLToPath(import.meta.url);
            const __dirname = path.dirname(__filename);"#,
            r#"import path from "node:path";
            import { fileURLToPath } from "node:url";
            const __filename = fileURLToPath(import.meta.url);
            const __dirname = import.meta.dirname;"#,
        ),
    ];

    Tester::new(PreferImportMetaProperties::NAME, PreferImportMetaProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
