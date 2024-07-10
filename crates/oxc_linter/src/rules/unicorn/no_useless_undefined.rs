use std::collections::hash_set::HashSet;

use itertools::Itertools;
use oxc_ast::ast::{Statement, VariableDeclarationKind};
use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::get_enclosing_function, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoUselessUndefinedConfig {
    check_arguments: bool,
    check_arrow_function_body: bool,
}

impl Default for NoUselessUndefinedConfig {
    fn default() -> Self {
        Self { check_arguments: true, check_arrow_function_body: true }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessUndefined(Box<NoUselessUndefinedConfig>);

impl std::ops::Deref for NoUselessUndefined {
    type Target = NoUselessUndefinedConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// undefined is the default value for new variables, parameters, return statements, etc… so specifying it doesn't make any difference.
    ///
    /// Where passing undefined as argument is required is due to bad TypeScript types in functions, in which case you can use checkArguments: false option.
    /// Using undefined as arrow function body sometimes make the purpose more explicit. You can use the checkArrowFunctionBody: false option to allow this.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    ///
    /// // Bad
    /// let foo = undefined;
    /// const {foo = undefined} = bar;
    /// const noop = () => undefined;
    /// function foo() {
    /// return undefined;
    /// }
    /// function foo(bar = undefined) { }
    /// function foo({bar = undefined}) { }
    /// foo(undefined);
    ///
    /// // Good
    /// let foo;
    /// const {foo} = bar;
    /// const noop = () => {};
    /// function foo() {
    ///	return;
    /// }
    /// function* foo() {
    ///     yield;
    /// }
    /// function foo(bar) { }
    /// function foo({bar}) { }
    /// foo();
    /// ```
    NoUselessUndefined,
    restriction,
);

fn no_useless_undefined_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-unicorn(no-useless-undefined): Do not use useless `undefined`",
    )
    .with_label(span0)
}

fn remove_undefined_and_equal_sign_if_necessary(
    span0: Span,
    ctx: &LintContext,
    remove_equal_sign: bool,
) -> String {
    let mut undefined_sequence_found = false;
    let mut stop_looking_for_spaces = false;
    let undefined_in_reverse = vec!['d', 'e', 'n', 'i', 'f', 'e', 'd', 'n', 'u'];
    let mut iter = undefined_in_reverse.into_iter().multipeek();
    String::from(ctx.source_range(span0))
        .chars()
        .rev()
        .filter(|c| {
            if remove_equal_sign && *c == '=' {
                return false;
            }
            if stop_looking_for_spaces {
                return true;
            }
            if undefined_sequence_found {
                if c.is_whitespace() {
                    return false;
                }
                stop_looking_for_spaces = true;
                return true;
            }
            if let Some(next) = iter.peek() {
                if c == next {
                    return true;
                }
            } else {
                undefined_sequence_found = true;
                if c.is_whitespace() {
                    return false;
                }
            }
            iter.reset_peek();
            true
        })
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>()
        .replace("undefined", "")
}

fn diagnostic_and_fix_statement(
    parent_span: Span,
    span0: Span,
    ctx: &LintContext,
    remove_equal_sign: bool,
) {
    ctx.diagnostic_with_fix(no_useless_undefined_diagnostic(span0), |fixer| {
        fixer.replace(
            parent_span,
            remove_undefined_and_equal_sign_if_necessary(parent_span, ctx, remove_equal_sign),
        )
    });
}

fn get_undefined_span_expression(expr_option: Option<&Expression>) -> (bool, Option<Span>) {
    if let Some(expr) = expr_option {
        let final_expr = expr.without_parenthesized();
        if let Expression::Identifier(ident) = final_expr {
            if ident.name == "undefined" {
                return (true, Some(ident.span));
            }
            return (false, Some(ident.span));
        }
    }
    (false, None)
}

fn check_if_authorized_method_name(name: &str) -> bool {
    let omitted_function_names: HashSet<&str> = HashSet::from([
        "is",
        "equal",
        "notEqual",
        "strictEqual",
        "notStrictEqual",
        "propertyVal",
        "notPropertyVal",
        "not",
        "include",
        "includes",
        "property",
        "toBe",
        "toHaveBeenCalledWith",
        "toContain",
        "toContainEqual",
        "toEqual",
        "same",
        "notSame",
        "strictSame",
        "strictNotSame",
        "push",
        "unshift",
        "add",
        "has",
        "set",
        "createContext",
        "ref",
    ]);

    if omitted_function_names.contains(name) {
        return true;
    }

    // handle set[A-Z] methods
    if name.starts_with("set") {
        return true;
    }

    false
}

impl Rule for NoUselessUndefined {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ReturnStatement(statement) => {
                if let (true, Some(span)) =
                    get_undefined_span_expression(statement.argument.as_ref())
                {
                    let function_may_be_undefined =
                        get_enclosing_function(node, ctx).map_or(false, |enclosing_function| {
                            match enclosing_function.kind() {
                                AstKind::Function(function_node) => {
                                    return function_node.return_type.as_ref().map_or(
                                        false,
                                        |return_type| {
                                            return_type.type_annotation.is_maybe_undefined()
                                        },
                                    );
                                }
                                AstKind::ArrowFunctionExpression(function_node) => {
                                    return function_node.return_type.as_ref().map_or(
                                        false,
                                        |return_type| {
                                            return_type.type_annotation.is_maybe_undefined()
                                        },
                                    );
                                }
                                _ => false,
                            }
                        });
                    if !function_may_be_undefined {
                        diagnostic_and_fix_statement(statement.span, span, ctx, false);
                    }
                }
            }
            AstKind::YieldExpression(statement) => {
                if statement.delegate {
                    return;
                }
                if let (true, Some(span)) =
                    get_undefined_span_expression(statement.argument.as_ref())
                {
                    diagnostic_and_fix_statement(statement.span, span, ctx, false);
                }
            }
            AstKind::VariableDeclaration(declaration) => {
                if declaration.kind == VariableDeclarationKind::Const {
                    return;
                }
                declaration.declarations.iter().for_each(|declarator| {
                    let ident_option =
                        declarator.init.as_ref().and_then(|init| init.get_identifier_reference());
                    if let Some(ident) = ident_option {
                        if ident.name == "undefined" {
                            diagnostic_and_fix_statement(declarator.span, ident.span, ctx, true);
                        }
                    }
                });
            }
            AstKind::CallExpression(statement) => {
                if !self.check_arguments {
                    return;
                }
                let mut function_name = statement
                    .callee
                    .as_member_expression()
                    .map_or("", |member| member.static_property_name().map_or("", |name| name));
                function_name = if function_name.is_empty() {
                    statement.callee.get_identifier_reference().map_or("", |ident| &ident.name)
                } else {
                    function_name
                };
                let is_authorized_method_name = check_if_authorized_method_name(function_name);
                if !is_authorized_method_name {
                    if function_name == "bind"
                        && statement.callee.is_member_expression()
                        && !statement.optional
                    {
                        if let (true, Some(span)) =
                            statement.arguments.first().map_or((false, None), |arg| {
                                get_undefined_span_expression(arg.as_expression())
                            })
                        {
                            diagnostic_and_fix_statement(statement.span, span, ctx, false);
                        }
                        return;
                    }
                    let undefined_args: Vec<(bool, Span)> = statement
                        .arguments
                        .iter()
                        .map(|arg| get_undefined_span_expression(arg.as_expression()))
                        .rev()
                        .take_while_inclusive(|arg| arg.0)
                        .map(|arg| (arg.0, arg.1.unwrap_or_default()))
                        .collect();
                    if let Some(last_undefined_arg_span) = undefined_args.first() {
                        if !last_undefined_arg_span.0 {
                            return;
                        }
                        let first_span = match undefined_args.last() {
                            Some(arg) => arg,
                            None => last_undefined_arg_span,
                        };
                        let last = if statement.span.end - 1 == last_undefined_arg_span.1.end {
                            last_undefined_arg_span.1.end
                        } else {
                            last_undefined_arg_span.1.end + 1
                        };
                        let first =
                            if first_span.0 { first_span.1.start } else { first_span.1.end };
                        let span_to_fix = Span::new(first, last);
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(span_to_fix),
                            |fixer| fixer.delete_range(span_to_fix),
                        );
                    }
                }
            }
            AstKind::ArrowFunctionExpression(arrow_fn) => {
                if self.check_arrow_function_body {
                    if arrow_fn.body.statements.len() > 1 {
                        return;
                    }
                    let is_return_type_undefined =
                        arrow_fn.return_type.as_ref().map_or(false, |return_type| {
                            return_type.type_annotation.is_maybe_undefined()
                        });
                    if is_return_type_undefined {
                        return;
                    }
                    let first_statement = arrow_fn.body.statements.first();
                    if let Some(Statement::ExpressionStatement(statement)) = first_statement {
                        if let (true, Some(span)) =
                            get_undefined_span_expression(Some(&statement.expression))
                        {
                            ctx.diagnostic_with_fix(
                                no_useless_undefined_diagnostic(span),
                                |fixer| fixer.replace(span, "{}"),
                            );
                        }
                    }
                }
            }
            AstKind::AssignmentPattern(assignment) => {
                if let (true, Some(span)) = get_undefined_span_expression(Some(&assignment.right)) {
                    diagnostic_and_fix_statement(assignment.span, span, ctx, true);
                }
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let mut cfg = NoUselessUndefinedConfig::default();

        if let Some(config) = value.get(0) {
            if let Some(val) = config.get("checkArguments").and_then(serde_json::Value::as_bool) {
                cfg.check_arguments = val;
            }

            if let Some(val) =
                config.get("checkArrowFunctionBody").and_then(serde_json::Value::as_bool)
            {
                cfg.check_arrow_function_body = val;
            }
        }

        Self(Box::new(cfg))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let options_ignore_arguments = json!([{ "checkArguments": false }]);
    let options_ignore_arrow_function_body = json!([{ "checkArrowFunctionBody": false }]);

    let pass = vec![
        ("function foo() {return;}", None, None, None),
        ("const foo = () => {};", None, None, None),
        ("let foo;", None, None, None),
        ("var foo;", None, None, None),
        ("const foo = undefined;", None, None, None),
        ("foo();", None, None, None),
        ("foo(bar,);", None, None, None),
        ("foo(undefined, bar);", None, None, None),
        ("const {foo} = {};", None, None, None),
        ("function foo({bar} = {}) {}", None, None, None),
        ("function foo(bar) {}", None, None, None),
        ("function* foo() {yield}", None, None, None),
        ("function* foo() {yield* undefined;}", None, None, None),
        ("if (Object.is(foo, undefined)){}", None, None, None),
        ("t.is(foo, undefined)", None, None, None),
        ("assert.equal(foo, undefined, message)", None, None, None),
        ("assert.notEqual(foo, undefined, message)", None, None, None),
        ("assert.strictEqual(foo, undefined, message)", None, None, None),
        ("assert.notStrictEqual(foo, undefined, message)", None, None, None),
        (r#"assert.propertyVal(foo, "bar", undefined, message)"#, None, None, None),
        (r#"assert.notPropertyVal(foo, "bar", undefined, message)"#, None, None, None),
        ("expect(foo).not(undefined)", None, None, None),
        (r#"expect(foo).to.have.property("bar", undefined)"#, None, None, None),
        ("expect(foo).toBe(undefined)", None, None, None),
        ("expect(foo).toContain(undefined)", None, None, None),
        ("expect(foo).toContainEqual(undefined)", None, None, None),
        ("expect(foo).toEqual(undefined)", None, None, None),
        ("t.same(foo, undefined)", None, None, None),
        ("t.notSame(foo, undefined)", None, None, None),
        ("t.strictSame(foo, undefined)", None, None, None),
        ("t.strictNotSame(foo, undefined)", None, None, None),
        ("expect(someFunction).toHaveBeenCalledWith(1, 2, undefined);", None, None, None),
        ("set.add(undefined);", None, None, None),
        ("map.set(foo, undefined);", None, None, None),
        ("array.push(foo, undefined);", None, None, None),
        ("array.push(undefined);", None, None, None),
        ("array.unshift(foo, undefined);", None, None, None),
        ("array.unshift(undefined);", None, None, None),
        ("createContext(undefined);", None, None, None),
        ("React.createContext(undefined);", None, None, None),
        ("setState(undefined)", None, None, None),
        ("setState?.(undefined)", None, None, None),
        ("props.setState(undefined)", None, None, None),
        ("props.setState?.(undefined)", None, None, None),
        ("array.includes(undefined)", None, None, None),
        ("set.has(undefined)", None, None, None),
        ("foo.bind(bar, undefined)", None, None, None),
        ("foo.bind(...bar, undefined)", None, None, None),
        ("foo.bind(...[], undefined)", None, None, None),
        ("foo.bind(...[undefined], undefined)", None, None, None),
        ("foo.bind(bar, baz, undefined)", None, None, None),
        ("foo?.bind(bar, undefined)", None, None, None),
        (
            "foo(undefined, undefined);",
            Some(serde_json::json!(options_ignore_arguments)),
            None,
            None,
        ),
        ("foo.bind(undefined);", Some(serde_json::json!(options_ignore_arguments)), None, None),
        (
            "const foo = () => undefined",
            Some(serde_json::json!(options_ignore_arrow_function_body)),
            None,
            None,
        ),
        ("prerenderPaths?.add(entry)", None, None, None),
        (
            r#"
            					function getThing(): string | undefined {
            						if (someCondition) {
            							return "hello world";
            						}

            						return undefined;
            					}
            				"#,
            None,
            None,
            None,
        ),
        (
            r#"
            					function getThing(): string | undefined {
            						if (someCondition) {
            							return "hello world";
            						} else if (anotherCondition) {
            							return undefined;
            						}

            						return undefined;
            					}
            				"#,
            None,
            None,
            None,
        ),
        ("const foo = (): undefined => {return undefined;}", None, None, None),
        ("const foo = (): undefined => undefined;", None, None, None),
        ("const foo = function (): undefined {return undefined}", None, None, None),
        ("export function foo(): undefined {return undefined}", None, None, None),
        (
            "
            					const object = {
            						method(): undefined {
            							return undefined;
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        (
            "
            					class A {
            						method(): undefined {
            							return undefined;
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        (
            "
            					const A = class A {
            						method(): undefined {
            							return undefined
            						}
            					};
            				",
            None,
            None,
            None,
        ),
        (
            "
            					class A {
            						static method(): undefined {
            							return undefined
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        (
            "
            					class A {
            						get method(): undefined {
            							return undefined;
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        (
            "
            					class A {
            						static get method(): undefined {
            							return undefined;
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        (
            "
            					class A {
            						#method(): undefined {
            							return undefined;
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        (
            "
            					class A {
            						private method(): undefined {
            							return undefined;
            						}
            					}
            				",
            None,
            None,
            None,
        ),
        ("createContext<T>(undefined);", None, None, None),
        ("React.createContext<T>(undefined);", None, None, None),
        (
            "
            					<script setup>
            					import * as vue from 'vue';
            					const foo = vue.ref(undefined);
            					</script>
            				",
            None,
            None,
            None,
        ),
        (
            "
            					<script setup>
            					import { ref } from 'vue';
            					const foo = ref(undefined);
            					</script>
            				",
            None,
            None,
            None,
        ),
    ];

    let fail = vec![
        ("function foo() {return undefined;}", None, None, None),
        ("const foo = () => undefined;", None, None, None),
        ("const foo = () => {return undefined;};", None, None, None),
        ("function foo() {return       undefined;}", None, None, None),
        ("function foo() {return /* comment */ undefined;}", None, None, None),
        ("function* foo() {yield undefined;}", None, None, None),
        ("function* foo() {yield                 undefined;}", None, None, None),
        ("let a = undefined;", None, None, None),
        ("let a = undefined, b = 2;", None, None, None),
        ("var a = undefined;", None, None, None),
        ("var a = undefined, b = 2;", None, None, None),
        ("var a = undefined, b = 2, c = undefined;", None, None, None),
        ("foo(undefined);", None, None, None),
        ("foo(undefined, undefined);", None, None, None),
        ("foo(undefined,);", None, None, None),
        ("foo(undefined, undefined,);", None, None, None),
        ("foo(bar, undefined);", None, None, None),
        ("foo(bar, undefined, undefined);", None, None, None),
        ("foo(undefined, bar, undefined);", None, None, None),
        ("foo(bar, undefined,);", None, None, None),
        ("foo(undefined, bar, undefined,);", None, None, None),
        ("foo(bar, undefined, undefined,);", None, None, None),
        ("foo(undefined, bar, undefined, undefined,);", None, None, None),
        (
            "
        					foo(
        						undefined,
        						bar,
        						undefined,
        						undefined,
        						undefined,
        						undefined,
        					)
        				",
            None,
            None,
            None,
        ),
        ("const {foo = undefined} = {};", None, None, None),
        ("const [foo = undefined] = [];", None, None, None),
        ("function foo(bar = undefined) {}", None, None, None),
        ("function foo({bar = undefined}) {}", None, None, None),
        ("function foo({bar = undefined} = {}) {}", None, None, None),
        ("function foo([bar = undefined]) {}", None, None, None),
        ("function foo([bar = undefined] = []) {}", None, None, None),
        ("return undefined;", None, None, None),
        (
            "
        					function foo():undefined {
        						function nested() {
        							return undefined;
        						}

        						return nested();
        					}
        				",
            None,
            None,
            None,
        ),
        (
            "
        				foo(
        					undefined,
        					bar,
        					undefined,
        					undefined,
        					undefined,
        					undefined,
        				)
        			",
            None,
            None,
            None,
        ),
        ("function foo([bar = undefined] = []) {}", None, None, None),
        ("foo(bar, undefined, undefined);", None, None, None),
        ("let a = undefined, b = 2;", None, None, None),
        (
            "
        				function foo() {
        					return /* */ (
        						/* */
        						(
        							/* */
        							undefined
        							/* */
        						)
        						/* */
        					) /* */ ;
        				}
        			",
            None,
            None,
            None,
        ),
        (
            "
        				function * foo() {
        					yield /* */ (
        						/* */
        						(
        							/* */
        							undefined
        							/* */
        						)
        						/* */
        					) /* */ ;
        				}
        			",
            None,
            None,
            None,
        ),
        (
            "
        				const foo = () => /* */ (
        					/* */
        					(
        						/* */
        						undefined
        						/* */
        					)
        					/* */
        				);
        			",
            None,
            None,
            None,
        ),
        ("foo.bind(undefined)", None, None, None),
        ("bind(foo, undefined)", None, None, None),
        ("foo.bind?.(bar, undefined)", None, None, None),
        ("foo[bind](bar, undefined)", None, None, None),
        ("foo.notBind(bar, undefined)", None, None, None),
        // (
        //     "
        // 				<script>
        // 				import {nextTick} from 'vue';
        // 				const foo = nextTick(undefined);
        // 				</script>
        // 			",
        //     None,
        //     None,
        //     None,
        // ), // kinda hard with how ast for JSX is generated
        ("function f(foo: Type = undefined) {}", None, None, None),
        ("function f(foo?: Type = undefined) {}", None, None, None),
        ("const f = function(foo: Type = undefined) {}", None, None, None),
        ("const f = (foo: Type = undefined) => {}", None, None, None),
        ("const f = {method(foo: Type = undefined){}}", None, None, None),
        ("const f = class {method(foo: Type = undefined){}}", None, None, None),
        ("function f(foo = undefined) {}", None, None, None),
        // ("function a({foo} = undefined) {}", None, None, Some(PathBuf::from("'foo.ts'"))), // I do not understand how this should work
    ];

    let fix = vec![
        ("function foo() {return undefined;}", "function foo() {return;}", None),
        ("const foo = () => undefined;", "const foo = () => {};", None),
        ("const foo = () => {return undefined;};", "const foo = () => {return;};", None),
        ("function foo() {return       undefined;}", "function foo() {return;}", None),
        (
            "function foo() {return /* comment */ undefined;}",
            "function foo() {return /* comment */;}",
            None,
        ),
        ("function* foo() {yield undefined;}", "function* foo() {yield;}", None),
        ("function* foo() {yield                 undefined;}", "function* foo() {yield;}", None),
        ("let a = undefined;", "let a;", None),
        ("let a = undefined, b = 2;", "let a, b = 2;", None),
        ("var a = undefined;", "var a;", None),
        ("var a = undefined, b = 2;", "var a, b = 2;", None),
        ("var a = undefined, b = 2, c = undefined;", "var a, b = 2, c;", None),
        ("foo(undefined);", "foo();", None),
        ("foo(undefined, undefined);", "foo();", None),
        ("foo(undefined,);", "foo();", None),
        ("foo(undefined, undefined,);", "foo();", None),
        ("foo(bar, undefined);", "foo(bar);", None),
        ("foo(bar, undefined, undefined);", "foo(bar);", None),
        ("foo(undefined, bar, undefined);", "foo(undefined, bar);", None),
        (
            "
        					foo(
        						undefined,
        						bar,
        						undefined,
        						undefined,
        						undefined,
        						undefined,
        					)
        				",
            "
        					foo(
        						undefined,
        						bar
        					)
        				",
            None,
        ),
        ("const {foo = undefined} = {};", "const {foo} = {};", None),
        ("const [foo = undefined] = [];", "const [foo] = [];", None),
        ("function foo(bar = undefined) {}", "function foo(bar) {}", None),
        ("function foo({bar = undefined}) {}", "function foo({bar}) {}", None),
        ("function foo({bar = undefined} = {}) {}", "function foo({bar} = {}) {}", None),
        ("function foo([bar = undefined]) {}", "function foo([bar]) {}", None),
        ("function foo([bar = undefined] = []) {}", "function foo([bar] = []) {}", None),
        ("return undefined;", "return;", None),
        (
            "
        					function foo():undefined {
        						function nested() {
        							return undefined;
        						}

        						return nested();
        					}
        				",
            "
        					function foo():undefined {
        						function nested() {
        							return;
        						}

        						return nested();
        					}
        				",
            None,
        ),
    ];
    Tester::new(NoUselessUndefined::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}

// #[test]
// fn weird_test_case() {
//     use crate::tester::Tester;
//     use std::path::PathBuf;

//     let pass = vec![
//         // ("const foo = (): string => undefined;", None, None, None), // I do not understand why it work this way un eslint for me this should yield an error
//         (
//             "
//             					<script>
//                                 import { ref } from 'vue';
//             					export default {
//                                     setup() {
//                                         const foo = ref(undefined);
//                                     }
//                                 }
//             					</script>
//             				",
//             None,
//             None,
//             None,
//         ),
//         /*
//             I think that here we have and error with the JSX parser because it generate this error on the setup() {}
//             Expected `}` but found `{`
//         */
//     ];

//     let fail = vec![
//         (
//             "
//         				<script>
//         				import {nextTick} from 'vue';
//         				const foo = nextTick(undefined);
//         				</script>
//         			",
//             None,
//             None,
//             None,
//         ), // kinda hard with how ast for JSX is generated
//         ("function a({foo} = undefined) {}", None, None, Some(PathBuf::from("'foo.ts'"))), // I do not understand how this should work
//     ];

//     let fix = vec![
//         // ("foo(bar, undefined,);", "foo(bar,);", None), // eslint version
//         ("foo(bar, undefined,);", "foo(bar);", None), // my solution does remove the last ','
//         // ("foo(undefined, bar, undefined,);", "foo(undefined, bar,);", None), eslint version
//         ("foo(undefined, bar, undefined,);", "foo(undefined, bar);", None), // my solution does remove the last ','
//         // ("foo(bar, undefined, undefined,);", "foo(bar,);", None), eslint version
//         ("foo(bar, undefined, undefined,);", "foo(bar);", None), // my solution does remove the last ','
//         // ("foo(undefined, bar, undefined, undefined,);", "foo(undefined, bar,);", None), // eslint version
//         ("foo(undefined, bar, undefined, undefined,);", "foo(undefined, bar);", None), // my solution does remove the last ','
//     ];
//     Tester::new(NoUselessUndefined::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
// }
