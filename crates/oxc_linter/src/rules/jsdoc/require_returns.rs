use oxc_ast::{
    ast::{MethodDefinitionKind, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;
use phf::phf_set;
use serde::Deserialize;

use crate::{
    ast_util::is_function_node,
    context::LintContext,
    rule::Rule,
    utils::{
        get_function_nearest_jsdoc_node, should_ignore_as_avoid, should_ignore_as_internal,
        should_ignore_as_private,
    },
    AstNode,
};

fn missing_returns_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("eslint-plugin-jsdoc(require-returns): Missing JSDoc `@returns` declaration for function.")
        .with_help("Add `@returns` tag to the JSDoc comment.")
        .with_labels([span0.into()])
}
fn duplicate_returns_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("eslint-plugin-jsdoc(require-returns): Duplicate `@returns` tags.")
        .with_help("Remove redundunt `@returns` tag.")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct RequireReturns(Box<RequireReturnsConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Requires that return statements are documented.
    /// Will also report if multiple `@returns` tags are present.
    ///
    /// ### Why is this bad?
    /// The rule is intended to prevent the omission of `@returns` tag when necessary.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** @returns Foo. */
    /// function quux () { return foo; }
    ///
    /// // Failing
    /// /** Foo. */
    /// function quux () { return foo; }
    /// /**
    ///  * @returns Foo!
    ///  * @returns Foo?
    ///  */
    /// function quux () { return foo; }
    /// ```
    RequireReturns,
    correctness,
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
fn default_true() -> bool {
    true
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

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_function_node(node) {
            return;
        }

        // TODO: Cover return Promise case
        // Ignore empty, or async functions
        let func_span = match node.kind() {
            AstKind::ArrowFunctionExpression(arrow_func) => {
                // If async, means always return Promise, skip
                if arrow_func.r#async {
                    return;
                }
                // If no expression, skip
                if !arrow_func.expression {
                    return;
                }

                arrow_func.span
            }
            AstKind::Function(func) if func.is_expression() || func.is_declaration() => {
                // If async, means always return Promise, skip
                if func.r#async {
                    return;
                }

                let Some(ref func_body) = func.body else {
                    return;
                };

                let mut return_found = None;
                for stmt in &func_body.statements {
                    if let Statement::ReturnStatement(ret) = stmt {
                        return_found = Some(ret);
                    }
                }

                // let Some(arg) = return_found.and_then(|ret| ret.argument) else {
                //     return;
                // };
                // arg;

                func.span
            }
            _ => return,
        };

        let Some(func_node) = get_function_nearest_jsdoc_node(node, ctx) else {
            return;
        };

        let config = &self.0;
        if let AstKind::MethodDefinition(def) = func_node.kind() {
            match def.kind {
                MethodDefinitionKind::Get => {
                    if !config.check_getters {
                        return;
                    }
                }
                MethodDefinitionKind::Constructor => {
                    if !config.check_constructors {
                        return;
                    }
                }
                _ => {}
            }
        }

        // If no JSDoc is found, skip
        let Some(jsdocs) = ctx.jsdoc().get_all_by_node(func_node) else {
            return;
        };

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
            return;
        }

        let jsdoc_tags = jsdocs.iter().flat_map(JSDoc::tags).collect::<Vec<_>>();
        let resolved_returns_tag_name = settings.resolve_tag_name("returns");

        if is_missing_returns_tag(&jsdoc_tags, &resolved_returns_tag_name) {
            ctx.diagnostic(missing_returns_diagnostic(func_span));
            return;
        }

        if let Some(span) = is_duplicated_returns_tag(&jsdoc_tags, &resolved_returns_tag_name) {
            ctx.diagnostic(duplicate_returns_diagnostic(span));
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
			         * @param ms time in millis
			         */
			        export const sleep = (ms: number) =>
			          new Promise<void>((res) => setTimeout(res, ms));
			      ",
            None,
            None,
        ),
        (
            "
			        /**
			         * @param ms time in millis
			         */
			        export const sleep = (ms: number) => {
			          return new Promise<void>((res) => setTimeout(res, ms));
			        };
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       *
			       * @returns The file contents as buffer.
			       */
			      export function readFixture(path: string): Promise<Buffer>;
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
			           *
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
			           *
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
			           *
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
			            *
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
			           *
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
			         * @param ms time in millis
			         */
			        export const sleep = (ms: number) =>
			          new Promise<string>((res) => setTimeout(res, ms));
			      ",
            None,
            None,
        ),
        (
            "
			        /**
			         * @param ms time in millis
			         */
			        export const sleep = (ms: number) => {
			          return new Promise<string>((res) => setTimeout(res, ms));
			        };
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Reads a test fixture.
			       */
			      export function readFixture(path: string): Promise<Buffer>;
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

    Tester::new(RequireReturns::NAME, pass, fail).test_and_snapshot();
}
