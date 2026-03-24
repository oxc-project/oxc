use crate::module_record::ImportImportName;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, BindingPattern, CallExpression, Expression, NewExpression,
        VariableDeclarator,
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
    filename_fns: FxHashSet<SymbolId>,
    /// SymbolIds that resolve to `dirname` from path/node:path
    dirname_fns: FxHashSet<SymbolId>,
    /// SymbolIds that are the url module object (default/namespace import)
    url_mods: FxHashSet<SymbolId>,
    /// SymbolIds that are the path module object (default/namespace import)
    path_mods: FxHashSet<SymbolId>,
    /// NodeIds of outer CallExpression for inline `process.getBuiltinModule("url").fileURLToPath(…)`
    inline_filename_calls: Vec<NodeId>,
    /// NodeIds of outer CallExpression for inline `process.getBuiltinModule("path").dirname(…)`
    inline_dirname_calls: Vec<NodeId>,
}

impl ImportSymbols {
    fn is_empty(&self) -> bool {
        self.filename_fns.is_empty()
            && self.dirname_fns.is_empty()
            && self.url_mods.is_empty()
            && self.path_mods.is_empty()
            && self.inline_filename_calls.is_empty()
            && self.inline_dirname_calls.is_empty()
    }
}

fn insert_module_obj(symbol_id: SymbolId, is_url: bool, sym: &mut ImportSymbols) {
    if is_url {
        sym.url_mods.insert(symbol_id);
    } else {
        sym.path_mods.insert(symbol_id);
    }
}

fn collect_relevant_symbols(ctx: &LintContext<'_>) -> ImportSymbols {
    let mut sym = ImportSymbols::default();

    // Static ESM imports via module_record.
    for entry in &ctx.module_record().import_entries {
        let source = entry.module_request.name();
        let is_url = URL_MODULES.contains(&source);
        let is_path = PATH_MODULES.contains(&source);
        if !is_url && !is_path {
            continue;
        }
        let Some(symbol_id) = ctx.scoping().get_root_binding(entry.local_name.name().into()) else {
            continue;
        };
        match &entry.import_name {
            ImportImportName::Name(name) => {
                let imported = name.name();
                if is_url && imported == "fileURLToPath" {
                    sym.filename_fns.insert(symbol_id);
                } else if is_path && imported == "dirname" {
                    sym.dirname_fns.insert(symbol_id);
                }
            }
            // import path from "…" or import * as url from "…"
            ImportImportName::Default(_) | ImportImportName::NamespaceObject => {
                insert_module_obj(symbol_id, is_url, &mut sym);
            }
        }
    }

    // Follow `process` unresolved references instead of walking all nodes.
    // `process` is a global — all usages appear in root_unresolved_references.
    let scoping = ctx.scoping();
    let Some(process_refs) = scoping.root_unresolved_references().get("process") else {
        return sym;
    };

    for &ref_id in process_refs {
        let reference = scoping.get_reference(ref_id);

        // process → parent must be StaticMemberExpression (process.getBuiltinModule)
        let member_node_id = ctx.nodes().parent_id(reference.node_id());
        let AstKind::StaticMemberExpression(member) = ctx.nodes().kind(member_node_id) else {
            continue;
        };
        if member.optional || member.property.name != "getBuiltinModule" {
            continue;
        }

        // process.getBuiltinModule → parent must be CallExpression (process.getBuiltinModule("…"))
        let call_node_id = ctx.nodes().parent_id(member_node_id);
        let AstKind::CallExpression(call) = ctx.nodes().kind(call_node_id) else { continue };
        let is_url = is_process_get_builtin_module_call(call, &URL_MODULES);
        let is_path = is_process_get_builtin_module_call(call, &PATH_MODULES);
        if !is_url && !is_path {
            continue;
        }

        // Parent of the call determines which form we have.
        let decl_node_id = ctx.nodes().parent_id(call_node_id);
        // const path = process.getBuiltinModule("…") or const { x } = process.getBuiltinModule("…")
        if let AstKind::VariableDeclarator(decl) = ctx.nodes().kind(decl_node_id) {
            collect_get_builtin_symbols(decl, &mut sym);
        } else {
            // Inline-chained: process.getBuiltinModule("…").fileURLToPath/dirname(…)
            // Parent is a StaticMemberExpression — navigate to the outer CallExpression.
            let AstKind::StaticMemberExpression(member) = ctx.nodes().kind(decl_node_id) else {
                continue;
            };
            if member.optional {
                continue;
            }
            let outer_call_id = ctx.nodes().parent_id(decl_node_id);
            if is_url && member.property.name == "fileURLToPath" {
                sym.inline_filename_calls.push(outer_call_id);
            } else if is_path && member.property.name == "dirname" {
                sym.inline_dirname_calls.push(outer_call_id);
            }
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
                    sym.filename_fns.insert(symbol_id);
                } else if is_path && key == "dirname" {
                    sym.dirname_fns.insert(symbol_id);
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
        if sym.is_empty() {
            return;
        }
        check_patterns(&sym, ctx);
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
            (&sym.filename_fns, "fileURLToPath", &sym.url_mods, URL_MODULES.as_slice())
        }
        ModuleKind::Dirname => {
            (&sym.dirname_fns, "dirname", &sym.path_mods, PATH_MODULES.as_slice())
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

/// Emits `import.meta.dirname` for every read reference to `symbol_id`
/// used as the sole argument to a dirname function call.
/// Returns true if any diagnostic was emitted.
fn emit_dirname_for_symbol(
    symbol_id: SymbolId,
    sym: &ImportSymbols,
    ctx: &LintContext<'_>,
) -> bool {
    let mut found = false;
    for reference in ctx.semantic().symbol_references(symbol_id).filter(|r| r.is_read()) {
        let ref_parent_id = ctx.nodes().parent_id(reference.node_id());
        let AstKind::CallExpression(call) = ctx.nodes().kind(ref_parent_id) else { continue };
        if is_single_arg_call(call)
            && is_callee_module_fn(&call.callee, ModuleKind::Dirname, sym, ctx)
        {
            ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(call.span, "dirname"), |fixer| {
                fixer.replace(call.span, "import.meta.dirname")
            });
            found = true;
        }
    }
    found
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

/// Returns true if `expr` is `import.meta.filename` directly, or an identifier
/// whose declaration or write site(s) are `import.meta.filename` assignments.
/// Does NOT match `fileURLToPath(…)` — those are handled by the filename leg.
fn is_meta_filename_arg(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    if is_import_meta_property(expr, "filename") {
        return true;
    }
    let Expression::Identifier(ident) = expr else { return false };
    let reference = ctx.scoping().get_reference(ident.reference_id());
    let Some(symbol_id) = reference.symbol_id() else { return false };
    // Check the declaration site (const __filename = import.meta.filename)
    let decl_node_id = ctx.scoping().symbol_declaration(symbol_id);
    if let AstKind::VariableDeclarator(d) = ctx.nodes().kind(decl_node_id)
        && d.init.as_ref().is_some_and(|e| is_import_meta_property(e, "filename"))
    {
        return true;
    }
    // Check later assignment sites (__filename = import.meta.filename)
    ctx.semantic().symbol_references(symbol_id).filter(|r| r.is_write()).any(|r| {
        let parent_id = ctx.nodes().parent_id(r.node_id());
        let rhs: Option<&Expression<'_>> = match ctx.nodes().kind(parent_id) {
            AstKind::AssignmentExpression(a) => Some(&a.right),
            _ => None,
        };
        rhs.is_some_and(|e| is_import_meta_property(e, "filename"))
    })
}

/// Given a confirmed `fileURLToPath(import.meta.url)` call, emit dirname or filename diagnostic.
fn handle_filename_call(
    call_id: NodeId,
    call: &CallExpression<'_>,
    sym: &ImportSymbols,
    ctx: &LintContext<'_>,
) {
    if !try_report_dirname(call_id, sym, ctx) {
        ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(call.span, "filename"), |fixer| {
            fixer.replace(call.span, "import.meta.filename")
        });
    }
}

/// Drive from known symbol sets and globals — no full node walk.
fn check_patterns(sym: &ImportSymbols, ctx: &LintContext<'_>) {
    // --- Filename leg: fileURLToPath(import.meta.url) → filename or dirname ---

    for &id in &sym.filename_fns {
        for reference in ctx.semantic().symbol_references(id).filter(|r| r.is_read()) {
            let call_id = ctx.nodes().parent_id(reference.node_id());
            let AstKind::CallExpression(call) = ctx.nodes().kind(call_id) else { continue };
            if !is_single_arg_call(call) {
                continue;
            }
            let Some(arg) = call.arguments.first().and_then(Argument::as_expression) else {
                continue;
            };
            if !is_import_meta_property(arg, "url") {
                continue;
            }
            handle_filename_call(call_id, call, sym, ctx);
        }
    }

    for &id in &sym.url_mods {
        for reference in ctx.semantic().symbol_references(id).filter(|r| r.is_read()) {
            let member_id = ctx.nodes().parent_id(reference.node_id());
            let AstKind::StaticMemberExpression(m) = ctx.nodes().kind(member_id) else { continue };
            if m.optional || m.property.name != "fileURLToPath" {
                continue;
            }
            let call_id = ctx.nodes().parent_id(member_id);
            let AstKind::CallExpression(call) = ctx.nodes().kind(call_id) else { continue };
            if !is_single_arg_call(call) {
                continue;
            }
            let Some(arg) = call.arguments.first().and_then(Argument::as_expression) else {
                continue;
            };
            if !is_import_meta_property(arg, "url") {
                continue;
            }
            handle_filename_call(call_id, call, sym, ctx);
        }
    }

    for &call_id in &sym.inline_filename_calls {
        let AstKind::CallExpression(call) = ctx.nodes().kind(call_id) else { continue };
        if !is_single_arg_call(call) {
            continue;
        }
        let Some(arg) = call.arguments.first().and_then(Argument::as_expression) else { continue };
        if !is_import_meta_property(arg, "url") {
            continue;
        }
        handle_filename_call(call_id, call, sym, ctx);
    }

    // --- URL global leg: new URL(import.meta.url) ---

    let scoping = ctx.scoping();
    if let Some(url_refs) = scoping.root_unresolved_references().get("URL") {
        for &ref_id in url_refs {
            let reference = scoping.get_reference(ref_id);
            let parent_id = ctx.nodes().parent_id(reference.node_id());
            let AstKind::NewExpression(new_expr) = ctx.nodes().kind(parent_id) else { continue };
            handle_new_url(new_expr, parent_id, sym, ctx);
        }
    }

    // --- Meta-filename leg: path.dirname(import.meta.filename) ---

    for &id in &sym.dirname_fns {
        for reference in ctx.semantic().symbol_references(id).filter(|r| r.is_read()) {
            let call_id = ctx.nodes().parent_id(reference.node_id());
            let AstKind::CallExpression(call) = ctx.nodes().kind(call_id) else { continue };
            if !is_single_arg_call(call) {
                continue;
            }
            let Some(arg) = call.arguments.first().and_then(Argument::as_expression) else {
                continue;
            };
            if is_meta_filename_arg(arg, ctx) {
                ctx.diagnostic_with_fix(
                    prefer_import_meta_diagnostic(call.span, "dirname"),
                    |fixer| fixer.replace(call.span, "import.meta.dirname"),
                );
            }
        }
    }

    for &id in &sym.path_mods {
        for reference in ctx.semantic().symbol_references(id).filter(|r| r.is_read()) {
            let member_id = ctx.nodes().parent_id(reference.node_id());
            let AstKind::StaticMemberExpression(m) = ctx.nodes().kind(member_id) else { continue };
            if m.optional || m.property.name != "dirname" {
                continue;
            }
            let call_id = ctx.nodes().parent_id(member_id);
            let AstKind::CallExpression(call) = ctx.nodes().kind(call_id) else { continue };
            if !is_single_arg_call(call) {
                continue;
            }
            let Some(arg) = call.arguments.first().and_then(Argument::as_expression) else {
                continue;
            };
            if is_meta_filename_arg(arg, ctx) {
                ctx.diagnostic_with_fix(
                    prefer_import_meta_diagnostic(call.span, "dirname"),
                    |fixer| fixer.replace(call.span, "import.meta.dirname"),
                );
            }
        }
    }

    for &call_id in &sym.inline_dirname_calls {
        let AstKind::CallExpression(call) = ctx.nodes().kind(call_id) else { continue };
        if !is_single_arg_call(call) {
            continue;
        }
        let Some(arg) = call.arguments.first().and_then(Argument::as_expression) else { continue };
        if is_meta_filename_arg(arg, ctx) {
            ctx.diagnostic_with_fix(prefer_import_meta_diagnostic(call.span, "dirname"), |fixer| {
                fixer.replace(call.span, "import.meta.dirname")
            });
        }
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
        1 => {
            let Some(arg) = new_expr.arguments[0].as_expression() else { return };
            if !is_import_meta_property(arg, "url") {
                return;
            }
            "filename"
        }
        2 => {
            let first_is_dot = matches!(
                &new_expr.arguments[0],
                Argument::StringLiteral(lit) if lit.value == "." || lit.value == "./"
            );
            if !first_is_dot {
                return;
            }
            let Some(arg) = new_expr.arguments[1].as_expression() else { return };
            if !is_import_meta_property(arg, "url") {
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
/// to `path.dirname(...)`, either directly or via a variable, and emits diagnostics.
///
/// Returns `true` if a dirname diagnostic was emitted (so callers can suppress a
/// redundant filename diagnostic for the same value).
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
        // Stored in variable: const __filename = …; path.dirname(__filename)
        AstKind::VariableDeclarator(var_decl) => {
            let BindingPattern::BindingIdentifier(bind) = &var_decl.id else { return false };
            emit_dirname_for_symbol(bind.symbol_id(), sym, ctx)
        }
        // Stored via assignment: __filename = …; path.dirname(__filename)
        AstKind::AssignmentExpression(assign) => {
            let AssignmentTarget::AssignmentTargetIdentifier(target) = &assign.left else {
                return false;
            };
            let reference = ctx.scoping().get_reference(target.reference_id());
            let Some(symbol_id) = reference.symbol_id() else { return false };
            emit_dirname_for_symbol(symbol_id, sym, ctx)
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
