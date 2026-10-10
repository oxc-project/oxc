use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, MemberExpression, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::{get_symbol_id_of_variable, is_method_call, variable_declaration_kind},
    context::LintContext,
    rule::Rule,
};

const ARRAY_METHODS_RETURNS_ARRAY: [&str; 13] = [
    "copyWithin",
    "fill",
    "filter",
    "flat",
    "flatMap",
    "map",
    "reverse",
    "sort",
    "splice",
    "toReversed",
    "toSorted",
    "toSpliced",
    "with",
];

/// Methods that exist on both `Array` and `String`.
/// See <https://github.com/sindresorhus/eslint-plugin-unicorn/issues/2216>
const ARRAY_OR_STRING_METHODS: [&str; 2] = ["concat", "slice"];

/// Maximum number of `const` initializers followed when resolving a `concat`/`slice` receiver.
const MAX_RESOLVE_DEPTH: u8 = 8;

fn prefer_set_has_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("should be a `Set`, and use `.has()` to check existence or non-existence.")
        .with_help("Switch to `Set`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSetHas;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `Set#has()` over `Array#includes()` when checking for existence or non-existence.
    ///
    /// ### Why is this bad?
    ///
    /// `Set#has()` is faster than `Array#includes()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const array = [1, 2, 3];
    /// const hasValue = value => array.includes(value);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const set = new Set([1, 2, 3]);
    /// const hasValue = value => set.has(value);
    /// ```
    /// ```js
    /// const array = [1, 2, 3];
    /// const hasOne = array.includes(1);
    /// ```
    PreferSetHas,
    unicorn,
    perf,
    dangerous_fix,
    version = "0.13.2",
    short_description = "Prefer `Set#has()` over `Array#includes()` when checking for existence or non-existence.",
);

fn is_array_of_or_from(callee: &MemberExpression) -> bool {
    callee.is_specific_member_access("Array", "of")
        || callee.is_specific_member_access("Array", "from")
}

/// Whether the receiver of a `concat`/`slice` call should be treated as an array.
/// String literals, and identifiers not initialized with an array, could be strings.
fn is_receiver_array(receiver: &Expression, ctx: &LintContext, depth: u8) -> bool {
    match receiver.without_parentheses() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => false,
        Expression::Identifier(ident) => {
            depth < MAX_RESOLVE_DEPTH && is_const_initialized_with_array(ident, ctx, depth + 1)
        }
        _ => true,
    }
}

fn is_const_initialized_with_array(
    ident: &IdentifierReference,
    ctx: &LintContext,
    depth: u8,
) -> bool {
    let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx) else { return false };
    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let AstKind::VariableDeclarator(declarator) = declaration.kind() else { return false };
    variable_declaration_kind(declarator, ctx).is_const()
        && declarator.id.is_binding_identifier()
        && declarator.init.as_ref().is_some_and(|init| is_kind_of_array_expr(init, ctx, depth))
}

fn is_kind_of_array_expr(expr: &Expression, ctx: &LintContext, depth: u8) -> bool {
    match expr {
        Expression::NewExpression(new_expr) => {
            new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "Array")
        }
        Expression::CallExpression(call_expr) => {
            let Some(callee) = call_expr.callee.get_member_expr() else {
                return call_expr.callee_name().is_some_and(|name| name == "Array");
            };

            if callee.is_computed() || callee.optional() {
                return false;
            }

            let Some(name) = callee.static_property_name() else { return false };

            is_array_of_or_from(callee)
                || ARRAY_METHODS_RETURNS_ARRAY.contains(&name)
                || (ARRAY_OR_STRING_METHODS.contains(&name)
                    && is_receiver_array(callee.object(), ctx, depth))
        }
        Expression::ArrayExpression(_) => true,
        _ => false,
    }
}

fn is_multiple_calls(node: &AstNode, ctx: &LintContext, root_scope_id: ScopeId) -> bool {
    let mut was_in_root_scope = node.scope_id() == root_scope_id;
    let mut is_multiple = false;
    for parent in ctx.nodes().ancestors(node.id()) {
        let parent_scope = parent.scope_id();
        if was_in_root_scope && parent_scope != root_scope_id {
            is_multiple = false;
            break;
        }
        was_in_root_scope = parent_scope == root_scope_id;
        let parent_kind = parent.kind();
        if matches!(
            parent_kind,
            AstKind::ForOfStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Function(_)
        ) {
            is_multiple = true;
            break;
        }
    }
    is_multiple
}

impl Rule for PreferSetHas {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };

        if variable_declaration_kind(declarator, ctx) == VariableDeclarationKind::Var {
            return;
        }

        let Some(init) = declarator.init.as_ref() else {
            return;
        };

        let Some(ident) = declarator.id.get_binding_identifier() else {
            return;
        };

        let symbol_id = ident.symbol_id();

        let module_record = ctx.module_record();
        if module_record.exported_bindings.contains_key(ident.name.as_str())
            || module_record.export_default.is_some_and(|default| default == ident.span)
        {
            return;
        }
        let symbol_table = ctx.scoping();

        let mut references = symbol_table.get_resolved_references(symbol_id).peekable();

        let Ok(len) = references.try_len() else {
            return;
        };
        if len == 0 {
            return;
        }

        let root_scope = node.scope_id();

        if len == 1 {
            let Some(reference) = references.peek() else {
                return;
            };

            let node = ctx.nodes().get_node(reference.node_id());

            if !is_multiple_calls(node, ctx, root_scope) {
                return;
            }
        }

        if references.any(|reference| {
            let node = ctx.nodes().get_node(reference.node_id());
            let parent_id = ctx.nodes().parent_id(node.id());
            let AstKind::CallExpression(call_expression) = ctx.nodes().parent_kind(parent_id)
            else {
                return true;
            };
            if call_expression.arguments.len() != 1 || call_expression.optional {
                return true;
            }
            let arg = &call_expression.arguments[0];
            if arg.is_spread() {
                return true;
            }
            let AstKind::StaticMemberExpression(member_expr) = ctx.nodes().parent_kind(node.id())
            else {
                return true;
            };
            if member_expr.optional {
                return true;
            }
            let is_method = is_method_call(
                call_expression,
                Some(&[ident.name.as_str()]),
                Some(&["includes"]),
                Some(1),
                Some(1),
            );
            !is_method
        }) {
            return;
        }

        // Checked last, as it may follow `const` initializers.
        if !is_kind_of_array_expr(init, ctx, 0) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_set_has_diagnostic(declarator.span), |fixer| {
            let fixer = fixer.for_multifix();
            let mut declaration_fix = fixer.new_fix_with_capacity(2);
            let new_string = "new";
            let set_start_string = "Set(";
            let set_end_string = ")";
            let set_array_string = format!("{set_start_string}[");
            let net_set_array_string = format!("{new_string} {set_array_string}");
            let new_set_start_string = format!("{new_string} {set_start_string}");
            let array_end_string = format!("]{set_end_string}");
            let array_parenthesis_len = 6; // Array( => 6
            let new_space_len = 4; // new  => 4
            match init {
                Expression::ArrayExpression(init_node) => {
                    declaration_fix
                        .push(fixer.insert_text_before(&init_node.span, new_set_start_string));
                    declaration_fix.push(fixer.insert_text_after(&init_node.span, set_end_string));
                }
                Expression::CallExpression(call_expr) => {
                    if call_expr.callee.is_identifier_reference() {
                        let start = call_expr.span.start;
                        let end = start + 6;
                        let span = Span::new(start, end);
                        declaration_fix.push(fixer.replace(span, net_set_array_string));
                        let start = call_expr.span.end - 1;
                        let end = start + 1;
                        let span = Span::new(start, end);
                        declaration_fix.push(fixer.replace(span, array_end_string));
                    } else {
                        declaration_fix
                            .push(fixer.insert_text_before(&call_expr.span, new_set_start_string));
                        declaration_fix
                            .push(fixer.insert_text_after(&call_expr.span, set_end_string));
                    }
                }
                Expression::NewExpression(new_expr) => {
                    let start = new_expr.span.start + new_space_len;
                    let end = start + array_parenthesis_len;
                    let span = Span::new(start, end);
                    declaration_fix.push(fixer.replace(span, set_array_string));
                    let start = new_expr.span.end - 1;
                    let end = start + 1;
                    let span = Span::new(start, end);
                    declaration_fix.push(fixer.replace(span, array_end_string));
                }
                _ => {}
            }

            let mut references_fix = fixer.new_fix_with_capacity(len);
            let references = symbol_table.get_resolved_references(symbol_id);
            for reference in references {
                let node = ctx.nodes().get_node(reference.node_id());
                let AstKind::StaticMemberExpression(member_expr) =
                    ctx.nodes().parent_kind(node.id())
                else {
                    continue;
                };
                let property_info = member_expr.static_property_info();
                references_fix.push(fixer.replace(property_info.0, "has"));
            }
            declaration_fix
                .extend(references_fix)
                .with_message("Switch expression to Set and update call to Set.has")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
            const foo = new Set([1, 2, 3]);
            function unicorn() {
                return foo.has(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            const isExists = foo.includes(1);
        ",
        "
            while (a) {
                const foo = [1, 2, 3];
                const isExists = foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            (() => {})(foo.includes(1));
        ",
        "
            foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const exists = foo.includes(1);
        ",
        "
            const exists = [1, 2, 3].includes(1);
        ",
        "
            const foo = [1, 2, 3];
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes;
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return includes(foo);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return bar.includes(foo);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo[includes](1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.indexOf(1) !== -1;
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                foo.includes(1);
                foo.length = 1;
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                if (foo.includes(1)) {}
                return foo;
                            }
        ",
        "
            var foo = [1, 2, 3];
            var foo = [4, 5, 6];
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = bar;
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes();
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1, 1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1, 0);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1, undefined);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(...[1]);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo?.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes?.(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo?.includes?.(1);
            }
        ",
        "
            function unicorn() {
                const foo = [1, 2, 3];
            }
            function unicorn2() {
                return foo.includes(1);
            }
        ",
        "
            export const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            module.exports = [1, 2, 3];
            function unicorn() {
                return module.exports.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            export {foo};
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            export default foo;
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            export {foo as bar};
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            module.exports = foo;
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            exports = foo;
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            module.exports.foo = foo;
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = NotArray(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = new NotArray(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = NotArray.from({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = NotArray.of(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array.notListed();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array[from]({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array[of](1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = 'Array'.from({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = 'Array'.of(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array['from']({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array['of'](1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = of(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = from({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = bar.notListed();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = _.map([1, 2, 3], value => value);
            function unicorn() {
                return _.includes(foo, 1);
            }
        ",
        "
            @connect(
                state => {
                    const availableComponents = ['is']
                    if (nsConfig.enabled) availableComponents.push('ns')
                    if (jsConfig.enabled) availableComponents.push('js')
                    if (asConfig.enabled) availableComponents.push('as')
                    return {
                        availableComponents,
                    }
                },
            )
            export default class A {}
        ",
        "
            @connect(
                state => {
                    const availableComponents = ['is']
                    if (nsConfig.enabled) availableComponents.push('ns')
                    if (jsConfig.enabled) availableComponents.push('js')
                    return {
                        availableComponents,
                    }
                },
            )
            export default class A {}
        ",
        // `concat` and `slice` can return a string
        // https://github.com/sindresorhus/eslint-plugin-unicorn/issues/2216
        "
            const foo = bar.concat();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = bar.slice();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const text = 'abc'.slice();
            text.includes('ab') || text.includes('bc');
        ",
        "
            const text = `abc`.concat('def');
            text.includes('ab') || text.includes('bc');
        ",
        "
            const text = `${a}bc`.slice();
            text.includes('ab') || text.includes('bc');
        ",
        "
            const items = 'abc';
            const foo = items.slice();
            foo.includes('ab') || foo.includes('bc');
        ",
        "
            let items = [1, 2, 3];
            items = 'abc';
            const foo = items.slice();
            foo.includes('ab') || foo.includes('bc');
        ",
        "
            const foo = Iterator.concat(bar);
            foo.includes(1) || foo.includes(2);
        ",
        "
            const prefix = 'hello world'.slice(0, 5);
            export function f(x) {
                return prefix.includes(x);
            }
        ",
        "
            const [items] = ['abc'];
            const foo = items.slice();
            foo.includes('ab') || foo.includes('bc');
        ",
        "
            const items = [1, 2, 3];
            function unicorn() {
                const items = 'abc';
                const foo = items.slice();
                return foo.includes('ab') || foo.includes('bc');
            }
        ",
        // Circular initializers must not recurse forever
        "
            const a = b.slice();
            const b = a.slice();
            const foo = a.slice();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        // Stop following `const` initializers after `MAX_RESOLVE_DEPTH`
        "
            const a0 = [1, 2, 3];
            const a1 = a0.slice(); const a2 = a1.slice(); const a3 = a2.slice();
            const a4 = a3.slice(); const a5 = a4.slice(); const a6 = a5.slice();
            const a7 = a6.slice(); const a8 = a7.slice(); const a9 = a8.slice();
            const foo = a9.slice();
            function unicorn() {
                return foo.includes(1);
            }
        ",
    ];

    let fail = vec![
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            const isExists = foo.includes(1);
            const isExists2 = foo.includes(2);
        ",
        "
            const foo = [1, 2, 3];
            for (const a of b) {
                foo.includes(1);
            }
        ",
        "
            async function unicorn() {
                const foo = [1, 2, 3];
                for await (const a of b) {
                    foo.includes(1);
                }
            }
        ",
        "
            const foo = [1, 2, 3];
            for (let i = 0; i < n; i++) {
                foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            for (let a in b) {
                foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            while (a)  {
                foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            do {
                foo.includes(1);
            } while (a)
        ",
        "
            const foo = [1, 2, 3];
            do {
                // …
            } while (foo.includes(1))
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function * unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            async function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            async function * unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            const unicorn = function () {
                return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            const unicorn = () => foo.includes(1);
        ",
        "
            const foo = [1, 2, 3];
            const a = {
                b() {
                    return foo.includes(1);
                }
            };
        ",
        "
            const foo = [1, 2, 3];
            class A {
                b() {
                    return foo.includes(1);
                }
            }
        ",
        "
            const foo = [...bar];
            function unicorn() {
                return foo.includes(1);
            }
            bar.pop();
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
                const exists = foo.includes(1);
                function isExists(find) {
                    return foo.includes(find);
                }
            }
        ",
        "
            function wrap() {
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1);
                }
            }
            const bar = [4, 5, 6];
            function unicorn() {
                return bar.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function wrap() {
                const exists = foo.includes(1);
                const bar = [1, 2, 3];
                function outer(find) {
                    const foo = [1, 2, 3];
                    while (a) {
                        foo.includes(1);
                    }
                    function inner(find) {
                        const bar = [1, 2, 3];
                        while (a) {
                            const exists = bar.includes(1);
                        }
                    }
                }
            }
        ",
        "
            const foo = Array(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = new Array(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array.from({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = Array.of(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const foo = _([1,2,3]);
            const bar = foo.map(value => value);
            function unicorn() {
                return bar.includes(1);
            }
        ",
        "
            const a: Array<'foo' | 'bar'> = ['foo', 'bar']

            for (let i = 0; i < 3; i++) {
                if (a.includes(someString)) {
                    console.log(123)
                }
            }
        ",
        "
            const foo = [1, 2, 3].slice();
            foo.includes(1) || foo.includes(2);
        ",
        "
            const foo = [1, 2, 3].concat(4);
            foo.includes(1) || foo.includes(2);
        ",
        "
            const items = [1, 2, 3];
            const foo = items.slice();
            foo.includes(1) || foo.includes(2);
        ",
        "
            const items = [1, 2, 3];
            const foo = items.concat(4);
            foo.includes(1) || foo.includes(2);
        ",
        "
            const items = [1, 2, 3];
            const copy = items.slice();
            const foo = copy.concat(4);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        "
            const items = 'abc';
            function unicorn() {
                const items = [1, 2, 3];
                const foo = items.slice();
                return foo.includes(1) || foo.includes(2);
            }
        ",
        "
            const foo = bar.baz.slice();
            function unicorn() {
                return foo.includes(1);
            }
        ",
    ];

    let fix = vec![
        (
            "
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2, 3]);
                function unicorn() {
                    return foo.has(1);
                }
            ",
        ),
        (
            "
                const foo = [...bar];
                function unicorn() {
                    return foo.includes(1);
                }
                bar.pop();
            ",
            "
                const foo = new Set([...bar]);
                function unicorn() {
                    return foo.has(1);
                }
                bar.pop();
            ",
        ),
        (
            "
                const foo = Array(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2]);
                function unicorn() {
                    return foo.has(1);
                }
            ",
        ),
        (
            "
                const foo = new Array(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2]);
                function unicorn() {
                    return foo.has(1);
                }
            ",
        ),
        (
            "
                const foo = _([1,2,3]);
                const bar = foo.map(value => value);
                function unicorn() {
                    return bar.includes(1);
                }
            ",
            "
                const foo = _([1,2,3]);
                const bar = new Set(foo.map(value => value));
                function unicorn() {
                    return bar.has(1);
                }
            ",
        ),
        (
            "
                const foo = Array.of(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set(Array.of(1, 2));
                function unicorn() {
                    return foo.has(1);
                }
            ",
        ),
        (
            "
                const foo = [1, 2, 3].slice();
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2, 3].slice());
                function unicorn() {
                    return foo.has(1);
                }
            ",
        ),
        (
            "
                const items = [1, 2, 3];
                const foo = items.concat(4);
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const items = [1, 2, 3];
                const foo = new Set(items.concat(4));
                function unicorn() {
                    return foo.has(1);
                }
            ",
        ),
    ];

    Tester::new(PreferSetHas::NAME, PreferSetHas::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
