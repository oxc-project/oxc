use std::cmp::max;

use oxc_allocator::ArenaBox;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    CallExpression, Expression, ObjectExpression, ObjectPropertyKind, PropertyKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_object_spread_diagnostic(span: Span, for_use_literal: bool) -> OxcDiagnostic {
    let help_message = if for_use_literal {
        "Use an object literal instead of `Object.assign`. eg: `{ foo: bar }`."
    } else {
        "Use an object spread instead of `Object.assign` eg: `{ ...foo }`."
    };
    OxcDiagnostic::warn("Disallow using `Object.assign` with an object literal as the first argument and prefer the use of object spread instead")
        .with_help(help_message)
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferObjectSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using `Object.assign` with an object literal as the first argument and prefer the use of object spread instead.
    ///
    /// ### Why is this bad?
    ///
    /// When `Object.assign` is called using an object literal as the first argument, this rule requires using the object spread syntax instead. This rule also warns on cases where an `Object.assign` call is made using a single argument that is an object literal, in this case, the `Object.assign` call is not needed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Object.assign({}, foo);
    ///
    /// Object.assign({}, {foo: 'bar'});
    ///
    /// Object.assign({ foo: 'bar'}, baz);
    ///
    /// Object.assign({}, baz, { foo: 'bar' });
    ///
    /// Object.assign({}, { ...baz });
    ///
    /// // Object.assign with a single argument that is an object literal
    /// Object.assign({});
    ///
    /// Object.assign({ foo: bar });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// ({ ...foo });
    ///
    /// ({ ...baz, foo: 'bar' });
    ///
    /// // Any Object.assign call without an object literal as the first argument
    /// Object.assign(foo, { bar: baz });
    ///
    /// Object.assign(foo, bar);
    ///
    /// Object.assign(foo, { bar, baz });
    ///
    /// Object.assign(foo, { ...baz });
    /// ```
    PreferObjectSpread,
    eslint,
    style,
    fix = fix_or_suggestion,
    version = "0.15.9",
    short_description = "Disallow using `Object.assign` with an object literal as the first argument and prefer the use of object spread instead.",
);

impl Rule for PreferObjectSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["assign"]), Some(1), None) {
            return;
        }

        let Some(callee) = call_expr.callee.as_member_expression() else {
            return;
        };

        match callee.object().get_inner_expression() {
            Expression::Identifier(ident) => {
                if ident.name != "Object" || !ctx.is_reference_to_global_variable(ident) {
                    return;
                }
            }
            Expression::StaticMemberExpression(member_expr) => {
                if let Expression::Identifier(ident) = member_expr.object.get_inner_expression() {
                    if ident.name != "globalThis" || !ctx.is_reference_to_global_variable(ident) {
                        return;
                    }
                } else {
                    return;
                }

                if member_expr.property.name != "Object" {
                    return;
                }
            }
            _ => return,
        }

        let arguments_len = call_expr.arguments.len();
        let mut has_non_literal_source = false;

        for (idx, arg) in call_expr.arguments.iter().enumerate() {
            let Some(Expression::ObjectExpression(obj_expr)) =
                arg.as_expression().map(Expression::get_inner_expression)
            else {
                if idx == 0 {
                    return;
                }

                if arg.is_spread() {
                    return;
                }

                has_non_literal_source = true;
                continue;
            };

            if arguments_len > 1 {
                if has_get_or_set_property(obj_expr) {
                    return;
                }

                if idx > 0 && has_proto_property(obj_expr) {
                    return;
                }
            }
        }

        let diagnostic = prefer_object_spread_diagnostic(call_expr.span(), arguments_len == 1);
        let fix = |fixer: RuleFixer<'_, 'a>| fix_object_assign(call_expr, node, ctx, fixer);

        if has_non_literal_source {
            ctx.diagnostic_with_suggestion(diagnostic, fix);
        } else {
            ctx.diagnostic_with_fix(diagnostic, fix);
        }
    }
}

fn fix_object_assign<'c, 'a: 'c>(
    call_expr: &CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    fixer: RuleFixer<'c, 'a>,
) -> RuleFix {
    let fixer = fixer.for_multifix();
    let mut rule_fixes = fixer.new_fix_with_capacity(2 + call_expr.arguments.len() * 5);

    let parent_kind = ctx.nodes().parent_kind(node.id());
    let needs_paren = !matches!(
        parent_kind,
        AstKind::VariableDeclarator(_)
            | AstKind::ArrayExpression(_)
            | AstKind::ReturnStatement(_)
            | AstKind::CallExpression(_)
            | AstKind::ObjectProperty(_)
            | AstKind::AssignmentExpression(_)
    );

    let Some(paren_offset) =
        ctx.find_next_token_within(call_expr.callee.span().end, call_expr.span.end, "(")
    else {
        return fixer.noop();
    };
    let callee_left_paren_span = Span::sized(call_expr.callee.span().end + paren_offset, 1);

    let (left, right) = if needs_paren { ("({", "})") } else { ("{", "}") };

    rule_fixes
        .push(fixer.replace(Span::new(call_expr.span.start, callee_left_paren_span.end), left));
    rule_fixes.push(fixer.replace(Span::new(call_expr.span.end - 1, call_expr.span.end), right));

    for arg in &call_expr.arguments {
        let Some(expression) = arg.as_expression() else {
            return fixer.noop();
        };

        if let Expression::ObjectExpression(obj_expr) = expression {
            let delete_span_of_left = get_delete_span_of_left(obj_expr, ctx);

            let delete_span_of_right = Span::new(
                max(get_delete_span_start_of_right(obj_expr, ctx), delete_span_of_left.end),
                obj_expr.span.end,
            );

            rule_fixes.push(fixer.delete_range(delete_span_of_left));
            rule_fixes.push(fixer.delete_range(delete_span_of_right));

            if (obj_expr.properties.is_empty()
                || ctx.source_range(get_last_char_span(expression, 1, ctx).unwrap()) == ",")
                && let Some(maybe_arg_comma_span) = get_char_span_after(expression, ctx)
                && ctx.source_range(maybe_arg_comma_span) == ","
            {
                rule_fixes.push(fixer.delete_range(maybe_arg_comma_span));
            }
        } else {
            let span = expression.span();
            let replacement = if matches!(
                expression,
                Expression::ArrowFunctionExpression(_)
                    | Expression::AssignmentExpression(_)
                    | Expression::ConditionalExpression(_)
            ) {
                format!("...({})", ctx.source_range(span))
            } else {
                format!("...{}", ctx.source_range(span))
            };

            rule_fixes.push(fixer.replace(span, replacement));
        }
    }

    rule_fixes.with_message("Replace with object spread")
}

fn has_get_or_set_property(obj_expr: &ObjectExpression) -> bool {
    obj_expr.properties.iter().any(|p| {
        let ObjectPropertyKind::ObjectProperty(p) = p else {
            return false;
        };

        p.kind == PropertyKind::Get || p.kind == PropertyKind::Set
    })
}

/// Returns `true` if the object expression has a `__proto__` property.
/// `__proto__:` in a source object literal does not create an own property, so
/// `Object.assign` does not copy it, but unwrapping it into the target literal
/// would set the target's prototype. Any other rewrite semantics (computed
/// `__["__proto__"]`, method `__proto__()`, etc.) also differ between copying
/// with `[[Set]]` and defining an own property, so they are treated the same.
fn has_proto_property(obj_expr: &ObjectExpression) -> bool {
    obj_expr.properties.iter().any(|p| {
        let ObjectPropertyKind::ObjectProperty(p) = p else {
            return false;
        };

        !p.shorthand && p.key.is_specific_static_name("__proto__")
    })
}

/**
 * Find the span of the first non-whitespace character before the expression.
 * (Includes character in the comment)
 */
fn get_char_span_before(start_char_span: Span, ctx: &LintContext) -> Option<Span> {
    let skip_count = start_char_span.start;
    let mut span_start = skip_count;
    for c in ctx.source_text()[..skip_count as usize].chars().rev() {
        let c_size = u32::try_from(c.len_utf8()).unwrap();
        span_start -= c_size;

        if c.is_whitespace() {
            continue;
        }

        let current_span = Span::sized(span_start, c_size);

        return Some(current_span);
    }

    None
}

fn get_last_char_span(expr: &Expression, last_from: u32, ctx: &LintContext) -> Option<Span> {
    let expr_span = expr.span();
    let mut count: u32 = 0;
    let mut span_start = expr_span.end;
    for c in ctx.source_range(expr_span).chars().rev() {
        let c_size = u32::try_from(c.len_utf8()).unwrap();
        span_start -= c_size;

        if c.is_whitespace() {
            continue;
        }

        let current_span = Span::sized(span_start, c_size);

        if ctx.comments().iter().any(|comment| comment.span.contains_inclusive(current_span)) {
            continue;
        }

        count += 1;
        if count > last_from {
            return Some(current_span);
        }
    }

    None
}

/**
 * Find the span of the first non-whitespace character after the expression.
 * And ignore characters in the comment.
 */
fn get_char_span_after(expr: &Expression, ctx: &LintContext) -> Option<Span> {
    let skip_count = expr.span().end;
    let mut span_end = skip_count;
    for c in ctx.source_text()[skip_count as usize..].chars() {
        let c_size = u32::try_from(c.len_utf8()).unwrap();
        span_end += c_size;

        if c.is_whitespace() {
            continue;
        }

        let current_span = Span::new(span_end - c_size, span_end);

        if ctx.comments().iter().any(|comment| comment.span.contains_inclusive(current_span)) {
            continue;
        }

        return Some(current_span);
    }

    None
}

fn get_delete_span_of_left(
    obj_expr: &ArenaBox<'_, ObjectExpression<'_>>,
    ctx: &LintContext,
) -> Span {
    let mut span_end = obj_expr.span.start;
    for (i, c) in ctx.source_range(obj_expr.span).char_indices() {
        if i != 0 && !c.is_whitespace() {
            break;
        }

        let c_size = u32::try_from(c.len_utf8()).unwrap();
        span_end += c_size;
    }

    Span::new(obj_expr.span.start, span_end)
}

fn get_delete_span_start_of_right(
    obj_expr: &ArenaBox<'_, ObjectExpression<'_>>,
    ctx: &LintContext,
) -> u32 {
    let obj_expr_last_char_span = Span::new(obj_expr.span.end - 1, obj_expr.span.end);
    let Some(prev_token_span) = get_char_span_before(obj_expr_last_char_span, ctx) else {
        return obj_expr_last_char_span.start;
    };

    let has_line_comment = if let Some(comment) =
        ctx.comments().iter().find(|&c| c.span.contains_inclusive(prev_token_span))
    {
        comment.is_line()
    } else {
        false
    };

    if has_line_comment {
        return obj_expr_last_char_span.start;
    }

    let mut span_start: u32 = obj_expr.span.end;
    for (i, c) in ctx.source_range(obj_expr.span).chars().rev().enumerate() {
        if i != 0 && !c.is_whitespace() {
            break;
        }

        let c_size = u32::try_from(c.len_utf8()).unwrap();
        span_start -= c_size;
    }

    span_start
}

#[test]
fn test() {
    use crate::fixer::FixKind;
    use crate::tester::Tester;

    let pass = vec![
        "Object.assign()",
        "let a = Object.assign(a, b)",
        "Object.assign(a, b)",
        "let a = Object.assign(b, { c: 1 })",
        "const bar = { ...foo }",
        "Object.assign(...foo)",
        "Object.assign(foo, { bar: baz })",
        "Object.assign/** comment😀 */(foo, { 'key😀': '😀😀' })", // with multi byte characters
        "Object.assign({}, ...objects)",
        "foo({ foo: 'bar' })",
        "
        const Object = {};
        Object.assign({}, foo);
        ",
        // "
        // Object = {};
        // Object.assign({}, foo);
        // ",
        "
        const Object = {};
        Object.assign({ foo: 'bar' });
        ",
        // "
        // Object = {};
        // Object.assign({ foo: 'bar' });
        // ",
        "
        const Object = require('foo');
        Object.assign({ foo: 'bar' });
        ",
        "
        import Object from 'foo';
        Object.assign({ foo: 'bar' });
        ",
        "
        import { Something as Object } from 'foo';
        Object.assign({ foo: 'bar' });
        ",
        "
        import { Object, Array } from 'globals';
        Object.assign({ foo: 'bar' });
        ",
        "
        var globalThis = foo;
        globalThis.Object.assign({}, foo)
        ", // { "ecmaVersion": 2020 },
        "class C { #assign; foo() { Object.#assign({}, foo); } }", // { "ecmaVersion": 2022 },
        "Object.assign({ get a() {} }, {})",
        "Object.assign({ set a(val) {} }, {})",
        "Object.assign({ get a() {} }, foo)",
        "Object.assign({ set a(val) {} }, foo)",
        "Object.assign({ foo: 'bar', get a() {}, baz: 'quux' }, quuux)",
        "Object.assign({ foo: 'bar', set a(val) {} }, { baz: 'quux' })",
        "Object.assign({}, { get a() {} })",
        "Object.assign({}, { set a(val) {} })",
        "Object.assign({}, { foo: 'bar', get a() {} }, {})",
        "Object.assign({ foo }, bar, {}, { baz: 'quux', set a(val) {}, quuux }, {})",
        "function f(MyObject, a) { const Object = MyObject; return Object.assign({}, a) }; Object.keys(x)",
        // `__proto__:` in a source literal sets the prototype but creates no own property, so
        // inlining it into the target literal changes semantics. Match ESLint and skip the call.
        "Object.assign({}, { __proto__: p })",
        "Object.assign({ foo: 'bar' }, { __proto__: p })",
        "Object.assign({}, { ['__proto__']: p })",
    ];

    let fail = vec![
        "Object.assign({}, foo)",
        "Object.assign  ({}, foo)",
        "Object.assign({}, { foo: 'bar' })",
        "Object.assign({}, baz, { foo: 'bar' })",
        "Object.assign({}, { foo: 'bar', baz: 'foo' })",
        "Object.assign/** comment😀 */({}, { '😀': '😀', '😆': '💪' })", // with multi byte characters
        "Object.assign({ foo: 'bar' }, baz)",
        "Object.assign({ foo: 'bar' }, cats, dogs, trees, birds)",
        "Object.assign({ foo: 'bar' }, Object.assign({ bar: 'foo' }, baz))",
        "Object.assign({ foo: 'bar' }, Object.assign({ bar: 'foo' }, Object.assign({}, { superNested: 'butwhy' })))",
        "Object.assign({foo: 'bar', ...bar}, baz)",
        "Object.assign({}, { foo, bar, baz })",
        "Object.assign({}, { [bar]: 'foo' })",
        "Object.assign({ ...bar }, { ...baz })",
        r#"
        Object.assign({ ...bar }, {
            // this is a bar
            foo: 'bar',
            baz: "cats"
        })
        "#,
        r#"
        Object.assign({
            boo: "lol",
            // I'm a comment
            dog: "cat"
        }, {
            // this is a bar
            foo: 'bar',
            baz: "cats"
        })
        "#,
        r#"
        const test = Object.assign({ ...bar }, {
            <!-- html comment
            foo: 'bar',
            baz: "cats"
            --> weird
        })
        "#, // {                "sourceType": "script"            },
        r#"
        const test = Object.assign({ ...bar }, {
            foo: 'bar', // inline comment
            baz: "cats"
        })
        "#,
        r#"
        const test = Object.assign({ ...bar }, {
            /**
             * foo
            */
            foo: 'bar',
            baz: "cats"
        })
        "#,
        "Object.assign({})",
        "Object.assign({ foo: bar })",
        "
        const foo = 'bar';
        Object.assign({ foo: bar })
        ",
        "
        foo = 'bar';
        Object.assign({ foo: bar })
        ",
        "let a = Object.assign({})",
        "let a = Object.assign({}, a)",
        "let a = Object.assign   ({}, a)",
        "let a = Object.assign({ a: 1 }, b)",
        "Object.assign(  {},  a,      b,   )",
        "Object.assign({}, a ? b : {}, b => c, a = 2)",
        "
        const someVar = 'foo';
        Object.assign({}, a ? b : {}, b => c, a = 2)
        ",
        "
        someVar = 'foo';
        Object.assign({}, a ? b : {}, b => c, a = 2)
        ",
        "[1, 2, Object.assign({}, a)]",
        "const foo = Object.assign({}, a)",
        "function foo() { return Object.assign({}, a) }",
        "foo(Object.assign({}, a));",
        "const x = { foo: 'bar', baz: Object.assign({}, a) }",
        "
        import Foo from 'foo';
        Object.assign({ foo: Foo });",
        "
        import Foo from 'foo';
        Object.assign({}, Foo);",
        "
        const Foo = require('foo');
        Object.assign({ foo: Foo });",
        "
        import { Something as somethingelse } from 'foo';
        Object.assign({}, somethingelse);
        ",
        "
        import { foo } from 'foo';
        Object.assign({ foo: Foo });
        ",
        "
        const Foo = require('foo');
        Object.assign({}, Foo);
        ",
        "
        const actions = Object.assign(
            {
                onChangeInput: this.handleChangeInput,
            },
            this.props.actions
        );
        ",
        "
        const actions = Object.assign(
            {
                onChangeInput: this.handleChangeInput, //
            },
            this.props.actions
        );
        ",
        "
        const actions = Object.assign(
            {
                onChangeInput: this.handleChangeInput //
            },
            this.props.actions
        );
        ",
        "
        const actions = Object.assign(
            (
                {
                    onChangeInput: this.handleChangeInput
                }
            ),
            (
                this.props.actions
            )
        );
        ",
        "eventData = Object.assign({}, eventData, { outsideLocality: `${originLocality} - ${destinationLocality}` })",
        "Object.assign({ });",
        "Object.assign({\n});",
        "globalThis.Object.assign({ });", // { "ecmaVersion": 2020 },
        "globalThis.Object.assign({\n});", // { "ecmaVersion": 2020 },
        "globalThis.Object.assign({}, foo)",
        "globalThis.Object.assign({}, { foo: 'bar' })", // { "ecmaVersion": 6 },
        "globalThis.Object.assign({}, baz, { foo: 'bar' })", // { "ecmaVersion": 2017 },
        "
        function foo () { var globalThis = bar; }
        globalThis.Object.assign({ });
        ", // { "ecmaVersion": 2020 },
        "
        const Foo = require('foo');
        globalThis.Object.assign({ foo: Foo });
        ", // { "ecmaVersion": 2020 },
        "Object.assign({ get a() {}, set b(val) {} })",
        "const obj = Object.assign<{}, Record<string, string[]>>({}, getObject());", // {                "parser": require("../../fixtures/parsers/typescript-parsers/object-assign-with-generic/object-assign-with-generic-1")            },
        "Object.assign<{}, A>({}, foo);", // {                "parser": require("../../fixtures/parsers/typescript-parsers/object-assign-with-generic/object-assign-with-generic-2")            }
    ];

    let fix = vec![
        // Non-literal sources are only offered as suggestions because the rewrite can change
        // observable semantics (getter evaluation order, own `__proto__` handling, etc.).
        ("Object.assign({}, foo)", "({ ...foo})", None, FixKind::Suggestion),
        ("Object.assign({}, src)", "({ ...src})", None, FixKind::Suggestion),
        // Literal-only rewrites remain safe automatic fixes.
        ("Object.assign({}, foo)", "({ ...foo})", None, FixKind::All),
        ("Object.assign  ({}, foo)", "({ ...foo})", None, FixKind::All),
        ("Object.assign({}, { foo: 'bar' })", "({ foo: 'bar'})", None, FixKind::Fix),
        ("Object.assign({}, baz, { foo: 'bar' })", "({ ...baz, foo: 'bar'})", None, FixKind::All),
        (
            "Object.assign({}, { foo: 'bar', baz: 'foo' })",
            "({ foo: 'bar', baz: 'foo'})",
            None,
            FixKind::All,
        ),
        (
            r"Object.assign/** comment with multi byte 😀 */({}, { '😀': '😀', '😆': '💪' })",
            r"({ '😀': '😀', '😆': '💪'})",
            None,
            FixKind::All,
        ),
        ("Object.assign({ foo: 'bar' }, baz)", "({foo: 'bar', ...baz})", None, FixKind::All),
        (
            "Object.assign({ foo: 'bar' }, cats, dogs, trees, birds)",
            "({foo: 'bar', ...cats, ...dogs, ...trees, ...birds})",
            None,
            FixKind::All,
        ),
        (
            "Object.assign({ foo: 'bar' }, Object.assign({ bar: 'foo' }, baz))",
            "({foo: 'bar', ...Object.assign({ bar: 'foo' }, baz)})",
            None,
            FixKind::All,
        ),
        (
            "Object.assign({ foo: 'bar' }, Object.assign({ bar: 'foo' }, Object.assign({}, { superNested: 'butwhy' })))",
            "({foo: 'bar', ...Object.assign({ bar: 'foo' }, Object.assign({}, { superNested: 'butwhy' }))})",
            None,
            FixKind::All,
        ),
        (
            "Object.assign({foo: 'bar', ...bar}, baz)",
            "({foo: 'bar', ...bar, ...baz})",
            None,
            FixKind::All,
        ),
        ("Object.assign({}, { foo, bar, baz })", "({ foo, bar, baz})", None, FixKind::All),
        ("Object.assign({}, { [bar]: 'foo' })", "({ [bar]: 'foo'})", None, FixKind::All),
        ("Object.assign({ ...bar }, { ...baz })", "({...bar, ...baz})", None, FixKind::All),
        (
            r#"
            Object.assign({ ...bar }, {
                // this is a bar
                foo: 'bar',
                baz: "cats"
            })
            "#,
            r#"
            ({...bar, // this is a bar
                foo: 'bar',
                baz: "cats"})
            "#,
            None,
            FixKind::All,
        ),
        (
            r#"
            Object.assign({
                boo: "lol",
                // I'm a comment
                dog: "cat"
                }, {
                // this is a bar
                foo: 'bar',
                baz: "cats"
            })
            "#,
            r#"
            ({boo: "lol",
                // I'm a comment
                dog: "cat", // this is a bar
                foo: 'bar',
                baz: "cats"})
            "#,
            None,
            FixKind::All,
        ),
        (
            r#"
            const test = Object.assign({ ...bar }, {
                /* comment
                foo: 'bar',
                baz: "cats"
                */ weird
            })
            "#,
            r#"
            const test = {...bar, /* comment
                foo: 'bar',
                baz: "cats"
                */ weird}
            "#,
            None,
            FixKind::All,
        ),
        (
            r#"
            const test = Object.assign({ ...bar }, {
                foo: 'bar', // inline comment
                baz: "cats"
            })
            "#,
            r#"
            const test = {...bar, foo: 'bar', // inline comment
                baz: "cats"}
            "#,
            None,
            FixKind::All,
        ),
        (
            r#"
            const test = Object.assign({ ...bar }, {
                /**
                 * foo
                 */
                foo: 'bar',
                baz: "cats"
            })
            "#,
            r#"
            const test = {...bar, /**
                 * foo
                 */
                foo: 'bar',
                baz: "cats"}
            "#,
            None,
            FixKind::All,
        ),
        ("Object.assign({})", "({})", None, FixKind::All),
        ("Object.assign({ foo: bar })", "({foo: bar})", None, FixKind::All),
        (
            "
            const foo = 'bar';
            Object.assign({ foo: bar })
            ",
            "
            const foo = 'bar';
            ({foo: bar})
            ",
            None,
            FixKind::All,
        ),
        (
            "
            foo = 'bar';
            Object.assign({ foo: bar })
            ",
            "
            foo = 'bar';
            ({foo: bar})
            ",
            None,
            FixKind::All,
        ),
        ("let a = Object.assign({})", "let a = {}", None, FixKind::All),
        ("let a = Object.assign({}, a)", "let a = { ...a}", None, FixKind::All),
        ("let a = Object.assign   ({}, a)", "let a = { ...a}", None, FixKind::All),
        ("let a = Object.assign({ a: 1 }, b)", "let a = {a: 1, ...b}", None, FixKind::All),
        ("Object.assign(  {},  a,      b,   )", "({    ...a,      ...b,   })", None, FixKind::All),
        (
            "Object.assign({}, a ? b : {}, b => c, a = 2)",
            "({ ...(a ? b : {}), ...(b => c), ...(a = 2)})",
            None,
            FixKind::All,
        ),
        (
            "
            const someVar = 'foo';
            Object.assign({}, a ? b : {}, b => c, a = 2)
            ",
            "
            const someVar = 'foo';
            ({ ...(a ? b : {}), ...(b => c), ...(a = 2)})
            ",
            None,
            FixKind::All,
        ),
        (
            "
            someVar = 'foo';
            Object.assign({}, a ? b : {}, b => c, a = 2)
            ",
            "
            someVar = 'foo';
            ({ ...(a ? b : {}), ...(b => c), ...(a = 2)})
            ",
            None,
            FixKind::All,
        ),
        ("[1, 2, Object.assign({}, a)]", "[1, 2, { ...a}]", None, FixKind::All),
        ("const foo = Object.assign({}, a)", "const foo = { ...a}", None, FixKind::All),
        (
            "function foo() { return Object.assign({}, a) }",
            "function foo() { return { ...a} }",
            None,
            FixKind::All,
        ),
        ("foo(Object.assign({}, a));", "foo({ ...a});", None, FixKind::All),
        (
            "const x = { foo: 'bar', baz: Object.assign({}, a) }",
            "const x = { foo: 'bar', baz: { ...a} }",
            None,
            FixKind::All,
        ),
        (
            "
            import Foo from 'foo';
            Object.assign({ foo: Foo });
            ",
            "
            import Foo from 'foo';
            ({foo: Foo});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            import Foo from 'foo';
            Object.assign({}, Foo);
            ",
            "
            import Foo from 'foo';
            ({ ...Foo});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const Foo = require('foo');
            Object.assign({ foo: Foo });
            ",
            "
            const Foo = require('foo');
            ({foo: Foo});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            import { Something as somethingelse } from 'foo';
            Object.assign({}, somethingelse);
            ",
            "
            import { Something as somethingelse } from 'foo';
            ({ ...somethingelse});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            import { foo } from 'foo';
            Object.assign({ foo: Foo });
            ",
            "
            import { foo } from 'foo';
            ({foo: Foo});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const Foo = require('foo');
            Object.assign({}, Foo);
            ",
            "
            const Foo = require('foo');
            ({ ...Foo});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const actions = Object.assign(
                {
                    onChangeInput: this.handleChangeInput,
                },
                this.props.actions
            );
            ",
            "
            const actions = {
                onChangeInput: this.handleChangeInput,
                ...this.props.actions
            };
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const actions = Object.assign(
                {
                    onChangeInput: this.handleChangeInput, //
                }, // comment 2
                this.props.actions
            );
            ",
            "
            const actions = {
                onChangeInput: this.handleChangeInput, //
                 // comment 2
                ...this.props.actions
            };
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const actions = Object.assign(
                {
                    onChangeInput: this.handleChangeInput //
                },
                this.props.actions
            );
            ",
            "
            const actions = {
                onChangeInput: this.handleChangeInput //
                ,
                ...this.props.actions
            };
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const actions = Object.assign(
                (
                    {
                        onChangeInput: this.handleChangeInput
                    }
                ),
                (
                    this.props.actions
                )
            );
            ",
            "
            const actions = {
                ...(
                    {
                        onChangeInput: this.handleChangeInput
                    }
                ),
                ...(
                    this.props.actions
                )
            };
            ",
            None,
            FixKind::All,
        ),
        (
            "eventData = Object.assign({}, eventData, { outsideLocality: `${originLocality} - ${destinationLocality}` })",
            "eventData = { ...eventData, outsideLocality: `${originLocality} - ${destinationLocality}`}",
            None,
            FixKind::All,
        ),
        ("Object.assign({ });", "({});", None, FixKind::All),
        ("Object.assign({\n});", "({});", None, FixKind::All),
        ("globalThis.Object.assign({ });", "({});", None, FixKind::All),
        ("globalThis.Object.assign({\n});", "({});", None, FixKind::All),
        (
            "
            function foo () { var globalThis = bar; }
            globalThis.Object.assign({ });
            ",
            "
            function foo () { var globalThis = bar; }
            ({});
            ",
            None,
            FixKind::All,
        ),
        (
            "
            const Foo = require('foo');
            globalThis.Object.assign({ foo: Foo });
            ",
            "
            const Foo = require('foo');
            ({foo: Foo});
            ",
            None,
            FixKind::All,
        ),
        (
            "Object.assign({ get a() {}, set b(val) {} })",
            "({get a() {}, set b(val) {}})",
            None,
            FixKind::All,
        ),
        (
            "const obj = Object.assign<{}, Record<string, string[]>>({}, getObject());",
            "const obj = { ...getObject()};",
            None,
            FixKind::All,
        ),
        ("Object.assign<{}, A>({}, foo);", "({ ...foo});", None, FixKind::All),
    ];
    Tester::new(PreferObjectSpread::NAME, PreferObjectSpread::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
