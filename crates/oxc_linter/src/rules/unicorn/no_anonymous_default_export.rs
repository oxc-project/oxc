use oxc_ast::{
    AstKind,
    ast::{AssignmentExpression, AssignmentTarget, ExportDefaultDeclarationKind, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, ast_util, context::LintContext, rule::Rule};

fn no_anonymous_default_export_diagnostic(span: Span, kind: ErrorNodeKind) -> OxcDiagnostic {
    let kind = match kind {
        ErrorNodeKind::Function => "function",
        ErrorNodeKind::Class => "class",
    };

    OxcDiagnostic::warn(format!("This {kind} default export is missing a name"))
        // TODO: suggest a name. https://github.com/sindresorhus/eslint-plugin-unicorn/blob/d3e4b805da31c6ed7275e2e2e770b6b0fbcf11c2/rules/no-anonymous-default-export.js#L41
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAnonymousDefaultExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows anonymous functions and classes as default exports.
    ///
    /// ### Why is this bad?
    ///
    /// Naming default exports improves searchability and ensures consistent
    /// identifiers for a module’s default export in both declaration and import.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export default class {}
    /// export default function () {}
    /// export default () => {};
    /// module.exports = class {};
    /// module.exports = function () {};
    /// module.exports = () => {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// export default class Foo {}
    /// export default function foo () {}
    ///
    /// const foo = () => {};
    /// export default foo;
    ///
    /// module.exports = class Foo {};
    /// module.exports = function foo () {};
    ///
    /// const foo = () => {};
    /// module.exports = foo;
    /// ```
    NoAnonymousDefaultExport,
    unicorn,
    restriction,
);

impl Rule for NoAnonymousDefaultExport {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let result = match node.kind() {
            // ESM: export default
            AstKind::ExportDefaultDeclaration(export_decl) => match &export_decl.declaration {
                ExportDefaultDeclarationKind::ClassDeclaration(class_decl)
                    if class_decl.id.is_none() =>
                {
                    Some((class_decl.span, ErrorNodeKind::Class))
                }
                ExportDefaultDeclarationKind::FunctionDeclaration(function_decl)
                    if function_decl.id.is_none() =>
                {
                    Some((function_decl.span, ErrorNodeKind::Function))
                }
                _ => {
                    export_decl.declaration.as_expression().and_then(is_anonymous_class_or_function)
                }
            },
            // CommonJS: module.exports
            AstKind::AssignmentExpression(expr)
                if is_top_expr(ctx, node) && is_common_js_export(expr) =>
            {
                is_anonymous_class_or_function(&expr.right)
            }
            _ => return,
        };

        if let Some((span, error_kind)) = result {
            ctx.diagnostic(no_anonymous_default_export_diagnostic(span, error_kind));
        }
    }
}

fn is_anonymous_class_or_function(expr: &Expression) -> Option<(Span, ErrorNodeKind)> {
    Some(match expr.get_inner_expression() {
        Expression::ClassExpression(expr) if expr.id.is_none() => (expr.span, ErrorNodeKind::Class),
        Expression::FunctionExpression(expr) if expr.id.is_none() => {
            (expr.span, ErrorNodeKind::Function)
        }
        Expression::ArrowFunctionExpression(expr) => (expr.span, ErrorNodeKind::Function),
        _ => return None,
    })
}

fn is_common_js_export(expr: &AssignmentExpression) -> bool {
    match &expr.left {
        AssignmentTarget::AssignmentTargetIdentifier(id) => id.name == "exports",
        AssignmentTarget::StaticMemberExpression(mem_expr) if !mem_expr.optional => {
            mem_expr.object.is_specific_id("module") && mem_expr.property.name == "exports"
        }
        _ => false,
    }
}

fn is_top_expr(ctx: &LintContext, node: &AstNode) -> bool {
    if !ctx.scoping().scope_flags(node.scope_id()).is_top() {
        return false;
    }

    let parent = ast_util::iter_outer_expressions(ctx.nodes(), node.id()).next();
    matches!(parent, Some(AstKind::ExpressionStatement(_)))
}

#[derive(Clone, Copy)]
enum ErrorNodeKind {
    Function,
    Class,
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = () => {}; export default foo;",
        r"module.exports = class Foo {};",
        r"module.exports = function foo () {};",
        r"const foo = () => {}; module.exports = foo;",
        ("export default function named() {}"),
        ("export default class named {}"),
        ("export default []"),
        ("export default {}"),
        ("export default 1"),
        ("export default false"),
        ("export default 0n"),
        ("notExports = class {}"),
        ("notModule.exports = class {}"),
        ("module.notExports = class {}"),
        ("module.exports.foo = class {}"),
        ("alert(exports = class {})"),
        ("foo = module.exports = class {}"),
    ];

    let fail = vec![
        ("export default function () {}"),
        ("export default class {}"),
        ("export default () => {}"),
        ("export default function * () {}"),
        ("export default async function () {}"),
        ("export default async function * () {}"),
        ("export default async () => {}"),
        ("export default class {}"),
        ("export default class extends class {} {}"),
        ("export default class{}"),
        ("export default class {}"),
        ("let Foo, Foo_, foo, foo_
			export default class {}"),
        ("let Foo, Foo_, foo, foo_
			export default (class{})"),
        ("export default (class extends class {} {})"),
        ("let Exports, Exports_, exports, exports_
			exports = class {}"),
        ("module.exports = class {}"),
        ("export default function () {}"),
        ("export default function* () {}"),
        ("export default async function* () {}"),
        ("export default async function*() {}"),
        ("export default async function *() {}"),
        ("export default async function   *   () {}"),
        ("export default async function * /* comment */ () {}"),
        ("export default async function * // comment
			() {}"),
        ("let Foo, Foo_, foo, foo_
			export default async function * () {}"),
        ("let Foo, Foo_, foo, foo_
			export default (async function * () {})"),
        ("let Exports, Exports_, exports, exports_
			exports = function() {}"),
        ("module.exports = function() {}"),
        ("export default () => {}"),
        ("export default async () => {}"),
        ("export default () => {};"),
        ("export default() => {}"),
        ("export default foo => {}"),
        ("export default (( () => {} ))"),
        ("/* comment 1 */ export /* comment 2 */ default /* comment 3 */  () => {}"),
        ("// comment 1
			export
			// comment 2
			default
			// comment 3
			() => {}"),
        ("let Foo, Foo_, foo, foo_
			export default async () => {}"),
        ("let Exports, Exports_, exports, exports_
			exports = (( () => {} ))"),
        ("// comment 1
			module
			// comment 2
			.exports
			// comment 3
			=
			// comment 4
			() => {};"),
        ("(( exports = (( () => {} )) ))"),
        ("(( module.exports = (( () => {} )) ))"),
        ("(( exports = (( () => {} )) ));"),
        ("(( module.exports = (( () => {} )) ));"),
        ("@decorator export default class {}"),
        ("export default @decorator(class {}) class extends class {} {}"),
        ("module.exports = @decorator(class {}) class extends class {} {}"),
        ("@decorator @decorator(class {}) export default class {}"),
    ];

    Tester::new(NoAnonymousDefaultExport::NAME, NoAnonymousDefaultExport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
