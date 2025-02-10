use oxc_ast::{
    ast::{BindingPatternKind, Expression, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;
use phf::phf_set;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        default_true, get_function_nearest_jsdoc_node, should_ignore_as_avoid,
        should_ignore_as_internal, should_ignore_as_private,
    },
};

fn missing_returns_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@returns` declaration for function.")
        .with_help("Add `@returns` tag to the JSDoc comment.")
        .with_label(span)
}
fn duplicate_returns_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Duplicate `@returns` tags.")
        .with_help("Remove redundant `@returns` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireReturns(Box<RequireReturnsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that return statements are documented.
    /// Will also report if multiple `@returns` tags are present.
    ///
    /// ### Why is this bad?
    ///
    /// The rule is intended to prevent the omission of `@returns` tag when necessary.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** Foo. */
    /// function quux () { return foo; }
    ///
    /// /**
    ///  * @returns Foo!
    ///  * @returns Foo?
    ///  */
    /// function quux () { return foo; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @returns Foo. */
    /// function quux () { return foo; }
    /// ```
    RequireReturns,
    jsdoc,
    pedantic,
);

#[derive(Debug, Clone, Deserialize)]
struct RequireReturnsConfig {
    #[serde(default = "default_exempted_by", rename = "exemptedBy")]
    exempted_by: Vec<String>,
    #[serde(default, rename = "checkConstructors")]
    check_constructors: bool,
    #[serde(default = "default_true", rename = "checkGetters")]
    check_getters: bool,
    #[serde(default, rename = "forceRequireReturn")]
    force_require_return: bool,
    #[serde(default, rename = "forceReturnsWithAsync")]
    force_returns_with_async: bool,
}
impl Default for RequireReturnsConfig {
    fn default() -> Self {
        Self {
            exempted_by: default_exempted_by(),
            check_constructors: false,
            check_getters: true,
            force_require_return: false,
            force_returns_with_async: false,
        }
    }
}
fn default_exempted_by() -> Vec<String> {
    vec!["inheritdoc".to_string()]
}

impl Rule for RequireReturns {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run_once(&self, ctx: &LintContext) {
        // Step 1. Collect functions to check
        // In this rule, existence of `return` statement differs the behavior.
        // Search `ReturnStatement` when visiting `Function` requires a lot of work.
        // Instead, we collect all functions and their attributes first.

        // Value of map: (AstNode, Span, Attrs: (isAsync, hasReturnValue))
        let mut functions_to_check = FxHashMap::default();
        'visit_node: for node in ctx.nodes() {
            match node.kind() {
                AstKind::Function(func) => {
                    functions_to_check.insert(node.id(), (node, func.span, (func.r#async, false)));
                }
                AstKind::ArrowFunctionExpression(arrow_func) => {
                    functions_to_check.insert(
                        node.id(),
                        (
                            node,
                            arrow_func.span,
                            if let Some(expr) = arrow_func.get_expression() {
                                is_promise_resolve_with_value(expr, ctx)
                                    .map_or((arrow_func.r#async, true), |v| (true, v))
                            } else {
                                (arrow_func.r#async, false)
                            },
                        ),
                    );
                }
                // Update function attrs entry with checking `return` value.
                // If syntax is valid, parent function node should be found by looking up the tree.
                //
                // It may not be accurate if there are multiple `return` in a function like:
                // ```js
                // function foo(x) {
                //   if (x) return Promise.resolve(1);
                //   return 2;
                // }
                // ```
                // IMO: This is a fault of the original rule design...
                AstKind::ReturnStatement(return_stmt) => {
                    let mut current_node = node;
                    while let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
                        match parent_node.kind() {
                            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                                // Ignore `return;`
                                let Some(argument) = &return_stmt.argument else {
                                    continue 'visit_node;
                                };

                                functions_to_check.entry(parent_node.id()).and_modify(|e| {
                                    // If `return somePromise` is found, treat this function as async
                                    match is_promise_resolve_with_value(argument, ctx) {
                                        Some(v) => e.2 = (true, v),
                                        None => e.2 = (e.2 .0, true),
                                    }
                                });
                                continue 'visit_node;
                            }
                            _ => {
                                current_node = parent_node;
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        // Step 2. Check collected functions
        for (node, func_span, (is_async, has_return_value)) in functions_to_check.values() {
            let Some(func_def_node) = get_function_nearest_jsdoc_node(node, ctx) else {
                continue;
            };
            // If no JSDoc is found, skip
            let Some(jsdocs) = ctx.jsdoc().get_all_by_node(func_def_node) else {
                continue;
            };

            let config = &self.0;
            let settings = &ctx.settings().jsdoc;
            // If JSDoc is found but safely ignored, skip
            if jsdocs
                .iter()
                .filter(|jsdoc| !should_ignore_as_custom_skip(jsdoc))
                .filter(|jsdoc| !should_ignore_as_avoid(jsdoc, settings, &config.exempted_by))
                .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
                .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
                .count()
                == 0
            {
                continue;
            }

            // If config disabled checking, skip
            if let AstKind::MethodDefinition(method_def) = func_def_node.kind() {
                match method_def.kind {
                    MethodDefinitionKind::Get => {
                        if !config.check_getters {
                            continue;
                        }
                    }
                    MethodDefinitionKind::Constructor => {
                        if !config.check_constructors {
                            continue;
                        }
                    }
                    _ => {}
                }
            }

            if !config.force_require_return &&
            // If sync, no return value, skip
            (!has_return_value && !is_async)
            {
                continue;
            }
            if !config.force_require_return && !config.force_returns_with_async &&
                // If async, no resolve value, skip
                (!has_return_value && *is_async)
            {
                continue;
            }

            let jsdoc_tags = jsdocs.iter().flat_map(JSDoc::tags).collect::<Vec<_>>();
            let resolved_returns_tag_name = settings.resolve_tag_name("returns");

            if is_missing_returns_tag(&jsdoc_tags, resolved_returns_tag_name) {
                ctx.diagnostic(missing_returns_diagnostic(*func_span));
                continue;
            }

            if let Some(span) = is_duplicated_returns_tag(&jsdoc_tags, resolved_returns_tag_name) {
                ctx.diagnostic(duplicate_returns_diagnostic(span));
            }
        }
    }
}

const CUSTOM_SKIP_TAG_NAMES: phf::Set<&'static str> = phf_set! {
    "abstract", "virtual", "class", "constructor", "type", "interface"
};
fn should_ignore_as_custom_skip(jsdoc: &JSDoc) -> bool {
    jsdoc.tags().iter().any(|tag| CUSTOM_SKIP_TAG_NAMES.contains(tag.kind.parsed()))
}

fn is_missing_returns_tag(jsdoc_tags: &[&JSDocTag], resolved_returns_tag_name: &str) -> bool {
    jsdoc_tags.iter().all(|tag| tag.kind.parsed() != resolved_returns_tag_name)
}

fn is_duplicated_returns_tag(
    jsdoc_tags: &Vec<&JSDocTag>,
    resolved_returns_tag_name: &str,
) -> Option<Span> {
    let mut returns_found = false;
    for tag in jsdoc_tags {
        if tag.kind.parsed() == resolved_returns_tag_name {
            if returns_found {
                return Some(tag.kind.span);
            }

            returns_found = true;
        }
    }

    None
}

/// - Some(true): `Promise` with value
/// - Some(false): `Promise` without value
/// - None: Not a `Promise` but some other expression
fn is_promise_resolve_with_value(expr: &Expression, ctx: &LintContext) -> Option<bool> {
    // `return new Promise(...)`
    if let Expression::NewExpression(new_expr) = expr {
        if new_expr.callee.is_specific_id("Promise") {
            return new_expr
                .arguments
                // Get `new Promise(HERE, ...)`
                .first()
                // Expect `new Promise(() => {})` or `new Promise(function() {})`
                .and_then(|arg| match arg.as_expression() {
                    Some(Expression::FunctionExpression(func)) => func.params.items.first(),
                    Some(Expression::ArrowFunctionExpression(arrow_func)) => {
                        arrow_func.params.items.first()
                    }
                    _ => None,
                })
                // Retrieve symbol_id of resolver, `new Promise((HERE, ...) => {})`
                .and_then(|first_param| match &first_param.pattern.kind {
                    BindingPatternKind::BindingIdentifier(ident) => Some(ident),
                    _ => None,
                })
                .and_then(|ident| {
                    // Find all usages of promise resolver
                    // It may not be accurate if there are multiple `resolve()` in a resolver like:
                    // ```js
                    // new Promise((resolve) => {
                    //   if (x) return resolve();
                    //   resolve(x)
                    // })
                    // ```
                    // IMO: This is a fault of the original rule design...
                    for resolve_ref in ctx.symbols().get_resolved_references(ident.symbol_id()) {
                        // Check if `resolve` is called with value
                        match ctx.nodes().parent_node(resolve_ref.node_id())?.kind() {
                            // `resolve(foo)`
                            AstKind::CallExpression(call_expr) => {
                                if !call_expr.arguments.is_empty() {
                                    return Some(true);
                                }
                            }
                            // `foo(resolve)`
                            AstKind::Argument(_) => {
                                return Some(true);
                            }
                            _ => continue,
                        }
                    }
                    None
                })
                .or(Some(false));
        }
    }

    // Tests do not cover `return Promise.xxx()`, but should be...?

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			          /**
			           * @returns Foo.
			           */
			          function quux () {

			            return foo;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (bar) {
			            bar.filter(baz => {
			              return baz.corge();
			            })
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns Array
			           */
			          function quux (bar) {
			            return bar.filter(baz => {
			              return baz.corge();
			            })
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns Array
			           */
			          const quux = (bar) => bar.filter(({ corge }) => corge())
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @inheritdoc
			           */
			          function quux (foo) {
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @override
			           */
			          function quux (foo) {
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @constructor
			           */
			          function quux (foo) {
			            return true;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @override
			           */
			          function quux (foo) {

			            return foo;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @class
			           */
			          function quux (foo) {
			            return true;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @constructor
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {object}
			           */
			          function quux () {

			            return {a: foo};
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {object}
			           */
			          const quux = () => ({a: foo});
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {object}
			           */
			          const quux = () => {
			            return {a: foo}
			          };
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          function quux () {
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          const quux = () => {

			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {undefined}
			           */
			          function quux () {
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {undefined}
			           */
			          const quux = () => {

			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const quux = () => {

			          }
			      ",
            None,
            None,
        ),
        (
            "
			      class Foo {
			        /**
			         *
			         */
			        constructor () {
			        }
			      }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			      const language = {
			        /**
			         * @param {string} name
			         */
			        set name(name) {
			          this._name = name;
			        }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          function quux () {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          function quux () {
			            return undefined;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          function quux () {
			            return undefined;
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          function quux () {
			            return;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {void}
			           */
			          function quux () {
			            return;
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /** @type {RequestHandler} */
			          function quux (req, res , next) {
			            return;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @returns {Promise}
			           */
			          async function quux () {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @returns {Promise}
			           */
			          async function quux () {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          async function quux () {}
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const quux = async function () {}
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const quux = async () => {}
			      ",
            None,
            None,
        ),
        (
            "
			      /** foo class */
			      class foo {
			        /** foo constructor */
			        constructor () {
			          // =>
			          this.bar = true;
			        }
			      }

			      export default foo;
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @type {MyCallback}
			           */
			          function quux () {

			          }
			      ",
            Some(serde_json::json!([
              {
                "exemptedBy": [
                  "type",
                ],
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * @param {array} a
			       */
			      async function foo(a) {
			        return;
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @function
			           */
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @callback
			           */
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @function
			           * @async
			           */
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @callback
			           * @async
			           */
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          class foo {
			            get bar() {
			              return 0;
			            }
			          }
			      ",
            Some(serde_json::json!([
              {
                "checkGetters": false,
              },
            ])),
            None,
        ),
        (
            "
			          class foo {
			            /** @returns zero */
			            get bar() {
			              return 0;
			            }
			          }
			      ",
            Some(serde_json::json!([
              {
                "checkGetters": true,
              },
            ])),
            None,
        ),
        (
            "
			          class foo {
			            /** @returns zero */
			            get bar() {
			              return 0;
			            }
			          }
			      ",
            Some(serde_json::json!([
              {
                "checkGetters": false,
              },
            ])),
            None,
        ),
        (
            "
			        class TestClass {
			          /**
			           *
			           */
			          constructor() { }
			        }
			        ",
            None,
            None,
        ),
        (
            "
			        class TestClass {
			          /**
			           * @returns A map.
			           */
			          constructor() {
			            return new Map();
			          }
			        }
			        ",
            None,
            None,
        ),
        (
            "
			        class TestClass {
			          /**
			           *
			           */
			          constructor() { }
			        }
			        ",
            Some(serde_json::json!([
              {
                "checkConstructors": false,
              },
            ])),
            None,
        ),
        (
            "
			      class TestClass {
			        /**
			         *
			         */
			        get Test() { }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      class TestClass {
			        /**
			         * @returns A number.
			         */
			        get Test() {
			          return 0;
			        }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      class TestClass {
			        /**
			         * pass(getter but config.checkGetters is false)
			         */
			        get Test() {
			          return 0;
			        }
			      }
			      ",
            Some(serde_json::json!([
              {
                "checkGetters": false,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              resolve();
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              setTimeout(() => {
			                resolve();
			              });
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              foo();
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              abc((resolve) => {
			                resolve(true);
			              });
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              abc(function (resolve) {
			                resolve(true);
			              });
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              if (true) {
			                resolve();
			              }
			            });
			            return;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise();
			          }
			      ",
            None,
            None,
        ),
        (
            "
			        /**
			         * Description.
			         */
			        async function foo() {
			          return new Promise(resolve => resolve());
			        }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       *
			       * @returns {void}.
			       */
			      export function readFixture(path: string): void;
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       */
			      export function readFixture(path: string): void;
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       */
			      export function readFixture(path: string);
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			          /**
			           * fail(no @returns)
			           */
			          function quux (foo) {

			            return foo;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {
			            someLabel: {
			              return foo;
			            }
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const foo = () => ({
			            bar: 'baz'
			          })
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const foo = bar=>({ bar })
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const foo = bar => bar.baz()
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return foo;
			          }
			      ",
            None,
            Some(serde_json::json!({ "settings": {
        "jsdoc": {
          "tagNamePreference": {
            "returns": "return",
          },
        },
      } })),
        ),
        (
            "
			          /**
			           * fail(forceRequireReturn: true)
			           */
			          async function quux() {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          const quux = async function () {}
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * fail(forceRequireReturn: true)
			           */
			          const quux = async () => {}
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			           /**
			            * fail(forceRequireReturn: true)
			            */
			           async function quux () {}
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			      const language = {
			        /**
			         * @param {string} name
			         */
			        get name() {
			          return this._name;
			        }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * fail(forceReturnsWithAsync: true)
			           */
			          async function quux () {
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @returns {undefined}
			           * @returns {void}
			           */
			          function quux (foo) {

			            return foo;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param foo
			           */
			          function quux (foo) {
			            return 'bar';
			          }
			      ",
            Some(serde_json::json!([
              {
                "exemptedBy": [
                  "notPresent",
                ],
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * @param {array} a
			       */
			      async function foo(a) {
			        return;
			      }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * @param {array} a
			       */
			      async function foo(a) {
			        return Promise.all(a);
			      }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			      class foo {
			        /** gets bar */
			        get bar() {
			          return 0;
			        }
			      }
			      ",
            Some(serde_json::json!([
              {
                "checkGetters": true,
              },
            ])),
            None,
        ),
        (
            "
			        class TestClass {
			          /**
			           *
			           */
			          constructor() {
			            return new Map();
			          }
			        }
			        ",
            Some(serde_json::json!([
              {
                "checkConstructors": true,
              },
            ])),
            None,
        ),
        (
            "
			      class TestClass {
			        /**
			         *
			         */
			        get Test() {
			          return 0;
			        }
			      }
			      ",
            Some(serde_json::json!([
              {
                "checkGetters": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              resolve(foo);
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              setTimeout(() => {
			                resolve(true);
			              });
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo) {

			            return new Promise(function (resolve, reject) {
			              foo(resolve);
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              while(true) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              do {
			                resolve(true);
			              }
			              while(true)
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              if (true) {
			                resolve(true);
			              }
			              return;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              if (resolve(true)) {
			                return;
			              }
			              return;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              if (true) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              true ? resolve(true) : null;
			              return;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            var a = {};
			            return new Promise((resolve, reject) => {
			              with (a) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            var a = {};
			            return new Promise((resolve, reject) => {
			              try {
			                resolve(true);
			              } catch (err) {}
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            var a = {};
			            return new Promise((resolve, reject) => {
			              try {
			              } catch (err) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            var a = {};
			            return new Promise((resolve, reject) => {
			              try {
			              } catch (err) {
			              } finally {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            var a = {};
			            return new Promise((resolve, reject) => {
			              switch (a) {
			              case 'abc':
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              if (true) {
			                resolve();
			              } else {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              for (let i = 0; i < 5 ; i++) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              for (const i of obj) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              for (const i in obj) {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              if (true) {
			                return;
			              } else {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              function a () {
			                resolve(true);
			              }
			              a();
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              return () => {
			                identifierForCoverage;
			                resolve(true);
			              };
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              a || resolve(true);
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              (r = resolve(true));
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              a + (resolve(true));
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              a, resolve(true);
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              +resolve();
			              [...resolve()];
			              [...+resolve(true)];
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise(function * (resolve, reject) {
			              yield resolve(true)
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise(async function (resolve, reject) {
			              await resolve(true)
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              someLabel: {
			                resolve(true);
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              var obj = {
			                [someKey]: 'val',
			                anotherKey: resolve(true)
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              var obj = {
			                [resolve(true)]: 'val',
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              `abc${resolve(true)}`;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              tagTemp`abc${resolve(true)}`;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              a.b[resolve(true)].c;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              abc?.[resolve(true)].d?.e(resolve(true));
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              const [a = resolve(true)] = arr;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              const {a = resolve(true)} = obj;
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              import(resolve(true));
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              class A {
			                method1 () {
			                  resolve();
			                }
			                @dec(function () {
			                  resolve()
			                })
			                method2 () {
			                  resolve(true);
			                }
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              class A {
			                method1 () {
			                  resolve();
			                }
			                @dec(function () {
			                  resolve(true)
			                })
			                method2 () {}
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              const a = class {
			                [b] () {
			                  resolve();
			                }
			                method1 () {
			                  resolve(true);
			                }
			                method2 () {}
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise((resolve, reject) => {
			              const a = class {
			                [b] () {
			                  resolve(true);
			                }
			              }
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          export function quux () {
			            return new Promise((resolve, reject) => {
			              resolve(true);
			            });
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux () {
			            return new Promise();
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          async function quux () {
			            return new Promise();
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          async function quux () {
			            return new Promise((resolve, reject) => {});
			          }
			      ",
            Some(serde_json::json!([
              {
                "forceReturnsWithAsync": true,
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       */
			      export function readFixture(path: string): void;
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       */
			      export function readFixture(path: string);
			      ",
            Some(serde_json::json!([
              {
                "forceRequireReturn": true,
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * @param {array} a
			       */
			      async function foo(a) {
			        return Promise.all(a);
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Description.
			       */
			      export default async function demo() {
			        return true;
			      }
			      ",
            None,
            None,
        ),
    ];

    Tester::new(RequireReturns::NAME, RequireReturns::PLUGIN, pass, fail).test_and_snapshot();
}
