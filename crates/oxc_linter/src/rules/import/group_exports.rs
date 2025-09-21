use oxc_ast::{
    AstKind,
    ast::{ImportOrExportKind, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule};

fn group_exports_diagnostic(span: Span, is_commonjs: bool) -> OxcDiagnostic {
    let (msg, help) = if is_commonjs {
        (
            "Multiple CommonJS exports; consolidate all exports into a single assignment to `module.exports`",
            "Combine multiple assignments into a single `module.exports = { ... }` statement",
        )
    } else {
        (
            "Multiple named export declarations; consolidate all named exports into a single export declaration",
            "Use a single export declaration with multiple specifiers: `export { spec1, spec2 }`",
        )
    };
    OxcDiagnostic::warn(msg).with_help(help).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct GroupExports;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports when named exports are not grouped together in a single export declaration
    /// or when multiple assignments to CommonJS module.exports
    /// or exports object are present in a single file.
    ///
    /// ### Why is this bad?
    ///
    /// An export declaration or module.exports assignment can appear anywhere in the code.
    /// By requiring a single export declaration all your exports will remain at one place,
    /// making it easier to see what exports a module provides.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export const first = true
    /// export const second = true
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const first = true
    /// const second = true
    /// export {
    ///     first,
    ///     second
    /// }
    /// ```
    GroupExports,
    import,
    style
);

impl Rule for GroupExports {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let semantic = ctx.semantic();
        let mut modules_source_record: FxHashMap<String, Vec<Span>> = FxHashMap::default();
        let mut modules_nodes = Vec::new();
        let mut type_source_record: FxHashMap<String, Vec<Span>> = FxHashMap::default();
        let mut type_nodes = Vec::new();
        let mut commonjs_nodes = Vec::new();

        for node in semantic.nodes() {
            match node.kind() {
                AstKind::ExportNamedDeclaration(export_decl) => match export_decl.export_kind {
                    ImportOrExportKind::Value => {
                        if let Some(source) = &export_decl.source {
                            modules_source_record
                                .entry(source.value.to_string())
                                .or_default()
                                .push(export_decl.span);
                        } else {
                            modules_nodes.push(export_decl.span);
                        }
                    }
                    ImportOrExportKind::Type => {
                        if let Some(source) = &export_decl.source {
                            type_source_record
                                .entry(source.value.to_string())
                                .or_default()
                                .push(export_decl.span);
                        } else {
                            type_nodes.push(export_decl.span);
                        }
                    }
                },
                AstKind::AssignmentExpression(assignment_expr) => {
                    let Some(member_expr) = assignment_expr.left.as_member_expression() else {
                        continue;
                    };
                    // e.g "exports.xxx = xxx;" or "module.exports = xxx;"
                    if member_expr.object().is_specific_id("exports")
                        || check_module_export(member_expr)
                    {
                        commonjs_nodes.push(assignment_expr.span);
                        continue;
                    }
                    // e.g "module.exports.xxx = xxx";
                    if let Some(obj_expr) = member_expr.object().as_member_expression()
                        && check_module_export(obj_expr)
                    {
                        commonjs_nodes.push(assignment_expr.span);
                    }
                }
                _ => {}
            }
        }

        if modules_nodes.len() > 1 {
            for item in &modules_nodes {
                ctx.diagnostic(group_exports_diagnostic(*item, false));
            }
        }
        if type_nodes.len() > 1 {
            for item in &type_nodes {
                ctx.diagnostic(group_exports_diagnostic(*item, false));
            }
        }
        if commonjs_nodes.len() > 1 {
            for item in &commonjs_nodes {
                ctx.diagnostic(group_exports_diagnostic(*item, true));
            }
        }
        for spans in modules_source_record.values() {
            if spans.len() > 1 {
                for item in spans {
                    ctx.diagnostic(group_exports_diagnostic(*item, false));
                }
            }
        }
        for spans in type_source_record.values() {
            if spans.len() > 1 {
                for item in spans {
                    ctx.diagnostic(group_exports_diagnostic(*item, false));
                }
            }
        }
    }
}

fn check_module_export(member_expr: &MemberExpression) -> bool {
    let Some(property_name) = member_expr.static_property_name() else {
        return false;
    };
    if member_expr.object().is_specific_id("module") && property_name == "exports" {
        return true;
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "export const valid = true",
        "
            export default {}
            export const test = true
        ",
        "
            const first = true
            const second = true
            export {
                first,
                second
            }
        ",
        "
            export default {}
            /* test */
            export const test = true
        ",
        "
            export default {}
            // test
            export const test = true
        ",
        "
            export const test = true
            /* test */
            export default {}
        ",
        "
            export { default as module1 } from './module-1'
            export { default as module2 } from './module-2'
        ",
        "
            module.exports = {}
        ",
        "
            module.exports = { test: true,
                another: false
            }
        ",
        "exports.test = true",
        "
            module.exports = {}
            const test = module.exports
        ",
        "
            exports.test = true
            const test = exports.test
        ",
        "
            module.exports = {}
            module.exports.too.deep = true
        ",
        "
            module.exports = {}
            exports.too.deep = true
        ",
        "
            export default {}
            const test = true
            export { test }
        ",
        "
            const test = true
            export { test }
            const another = true
            export default {}
        ",
        "
            module.something.else = true
            module.something.different = true
        ",
        "
            module.exports.test = true
            module.something.different = true
        ",
        "
            exports.test = true
            module.something.different = true
        ",
        "
            unrelated = 'assignment'
            module.exports.test = true
        ",
        "
             type firstType = {
                propType: string
            };
            const first = {};
            export type { firstType };
            export { first };
        ",
        "
            type firstType = {
                propType: string
            };
            type secondType = {
                propType: string
            };
            export type { firstType, secondType };
        ",
        "
            export type { type1A, type1B } from './module-1'
            export { method1 } from './module-1';
        ",
    ];

    let fail = vec![
        "
            export const first = true;
            export const second = true;
            export type A = {};
            export type { B } from 'b';
            export { module1 } from 'module-1';
            export { module2 } from 'module-2';
            export * as ttt from 'na';
            export * from 'star';
            const t = 3;
            export { t };
        ",
        "
            export type A = {
                name: string
            }
            export type B = {
                name: string
            };
        ",
        "
            exports.first = true
            exports.second = true
        ",
        "
            exports.second = true
            module.exports = {
            }
        ",
        "
            export type { type1 } from './module-1'
            export type { type2 } from './module-1'
        ",
        "
            type firstType = {
            propType: string
            };
            type secondType = {
            propType: string
            };
            const first = {};
            export type { firstType };
            export type { secondType };
            export { first };
        ",
        r#"
            module.exports = "non-object"
            module.exports.attached = true
            module.exports.another = true
        "#,
        r#"
            module.exports = "non-object"
            module.exports.attached = true
        "#,
        "
            module.exports = () => {}
            exports.test = true
            exports.another = true
        ",
        "
            module.exports = function test() {}
            module.exports.attached = true
        ",
        "
            module.exports = () => {}
            module.exports.attached = true
        ",
        "
            exports.test = true
            module.exports.another = true
        ",
        "
            module.exports.test = true
            module.exports.another = true
        ",
        "
            module.exports = { test: true }
            module.exports.another = true
        ",
        "
            module.exports = {}
            module.exports.test = true
        ",
        "
            export { method1 } from './module-1'
            export { method2 } from './module-1'
        ",
        "
            export default a = 3;
            export const b = 2;
            export const c = 2;
        ",
    ];

    Tester::new(GroupExports::NAME, GroupExports::PLUGIN, pass, fail).test_and_snapshot();
}
