use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(require-yields): Missing `@yields` declaration.")]
#[diagnostic(severity(warning), help("Add `@yields` tag to the JSDoc comment."))]
struct RequireYieldsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireYields(Box<RequireYieldsConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Requires that yields are documented.
    /// Will also report if multiple `@yields` tags are present.
    ///
    /// ### Why is this bad?
    /// The rule is intended to prevent the omission of `@yields` tags when they are necessary.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** * @yields Foo */
    /// function * quux (foo) { yield foo; }
    ///
    /// // Failing
    /// function * quux (foo) { yield foo; }
    /// /**
    ///  * @yields {undefined}
    ///  * @yields {void}
    ///  */
    /// function * quux (foo) {}
    /// ```
    RequireYields,
    correctness
);

#[derive(Debug, Clone, Deserialize)]
struct RequireYieldsConfig {
    #[serde(default, rename = "withGeneratorTag")]
    with_generator_tag: bool,
    #[serde(default = "default_true", rename = "forceRequireYields")]
    force_require_yields: bool,
    #[serde(default = "default_exempted_by", rename = "exemptedBy")]
    exempted_by: Vec<String>,
}
impl Default for RequireYieldsConfig {
    fn default() -> Self {
        Self {
            with_generator_tag: false,
            force_require_yields: true,
            exempted_by: default_exempted_by(),
        }
    }
}

fn default_exempted_by() -> Vec<String> {
    vec!["inheritdoc".to_string()]
}
fn default_true() -> bool {
    true
}

impl Rule for RequireYields {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::YieldExpression(yield_expression) = node.kind() else {
            return;
        };

        let config = &self.0;
        println!("👻 {config:#?}");
        let settings = &ctx.settings().jsdoc;
        println!("👻 {settings:#?}");

        ctx.diagnostic(RequireYieldsDiagnostic(Span::default()));

        // if config.forceRequireYields {
        //
        // } else {
        //   if node != YieldExpression { return }
        //   if !yieldValue { return }
        //   const functionNode = parent(node);
        // }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
        			          /**
        			           * @yields Foo.
        			           */
        			          function * quux () {
			
        			            yield foo;
        			          }
        			      ",
            None,
            None,
        ),
        // (
        //     "
        // 			          /**
        // 			           *
        // 			           */
        // 			          function * quux () {
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           *
        // 			           */
        // 			          function * quux () {
        // 			            yield;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           *
        // 			           */
        // 			          function quux (bar) {
        // 			            bar.doSomething(function * (baz) {
        // 			              yield baz.corge();
        // 			            })
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {Array}
        // 			           */
        // 			          function * quux (bar) {
        // 			            yield bar.doSomething(function * (baz) {
        // 			              yield baz.corge();
        // 			            })
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @inheritdoc
        // 			           */
        // 			          function * quux (foo) {
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @override
        // 			           */
        // 			          function * quux (foo) {
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @constructor
        // 			           */
        // 			          function * quux (foo) {
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @implements
        // 			           */
        // 			          function * quux (foo) {
        // 			            yield;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @override
        // 			           */
        // 			          function * quux (foo) {

        // 			            yield foo;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @class
        // 			           */
        // 			          function * quux (foo) {
        // 			            yield foo;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {object}
        // 			           */
        // 			          function * quux () {

        // 			            yield {a: foo};
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function * quux () {
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {undefined}
        // 			           */
        // 			          function * quux () {
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function quux () {
        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function * quux () {
        // 			            yield undefined;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function * quux () {
        // 			            yield undefined;
        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function * quux () {
        // 			            yield;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function * quux () {
        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {void}
        // 			           */
        // 			          function * quux () {
        // 			            yield;
        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /** @type {SpecialIterator} */
        // 			          function * quux () {
        // 			            yield 5;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields {Something}
        // 			           */
        // 			          async function * quux () {
        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           *
        // 			           */
        // 			          async function * quux () {}
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           *
        // 			           */
        // 			          const quux = async function * () {}
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @type {MyCallback}
        // 			           */
        // 			          function * quux () {
        // 			            yield;
        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "exemptedBy": [
        //           "type",
        //         ],
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			      /**
        // 			       * @param {array} a
        // 			       */
        // 			      async function * foo (a) {
        // 			        yield;
        // 			      }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @function
        // 			           */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @callback
        // 			           */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "forceRequireYields": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @generator
        // 			           */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "withGeneratorTag": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @generator
        // 			           * @yields
        // 			           */
        //                 *function() {}
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "withGeneratorTag": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @yields
        // 			           */
        // 			          function * quux (foo) {

        // 			            const a = yield foo;
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           *
        // 			           */
        // 			          function * quux (foo) {
        // 			            const a = function * bar () {
        // 			              yield foo;
        // 			            }
        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
    ];

    let fail = vec![
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux (foo) {

      // 			            yield foo;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux (foo) {
      // 			            someLabel: {
      // 			              yield foo;
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux (foo) {

      // 			            const a = yield foo;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux (foo) {
      // 			            yield foo;
      // 			          }
      // 			      ",
      //       None,
      //       Some(serde_json::json!({ "settings": {
      //   "jsdoc": {
      //     "tagNamePreference": {
      //       "yields": "yield",
      //     },
      //   },
      // } })),
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux() {
      // 			            yield 5;
      // 			          }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux() {
      // 			            yield;
      // 			          }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          const quux = async function * () {
      // 			            yield;
      // 			          }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			           /**
      // 			            *
      // 			            */
      // 			           async function * quux () {
      // 			             yield;
      // 			           }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            yield;
      // 			          }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           * @function
      // 			           * @generator
      // 			           */
      //                   *function() {}
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           * @yields {undefined}
      // 			           * @yields {void}
      // 			           */
      // 			          function * quux (foo) {

      // 			            return foo;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           * @param foo
      // 			           */
      // 			          function * quux (foo) {
      // 			            yield 'bar';
      // 			          }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "exemptedBy": [
      //             "notPresent",
      //           ],
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			      /**
      // 			       * @param {array} a
      // 			       */
      // 			      async function * foo(a) {
      // 			        return;
      // 			      }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			      /**
      // 			       * @param {array} a
      // 			       */
      // 			      async function * foo(a) {
      // 			        yield Promise.all(a);
      // 			      }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			      class quux {
      // 			        /**
      // 			         *
      // 			         */
      // 			        * quux () {
      // 			          yield;
      // 			        }
      // 			      }
      // 			      ",
      //       Some(serde_json::json!([
      //         {
      //           "forceRequireYields": true,
      //         },
      //       ])),
      //       None,
      //   ),
      //   (
      //       "
      // 			      /**
      // 			       * @param {array} a
      // 			       */
      // 			      async function * foo(a) {
      // 			        yield Promise.all(a);
      // 			      }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            if (true) {
      // 			              yield;
      // 			            }
      // 			            yield true;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            if (yield false) {

      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            b ? yield false : true
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            try {
      // 			              yield true;
      // 			            } catch (err) {
      // 			            }
      // 			            yield;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            try {
      // 			            } finally {
      // 			              yield true;
      // 			            }
      // 			            yield;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            try {
      // 			              yield;
      // 			            } catch (err) {
      // 			            }
      // 			            yield true;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            try {
      // 			              something();
      // 			            } catch (err) {
      // 			              yield true;
      // 			            }
      // 			            yield;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            switch (true) {
      // 			            case 'abc':
      // 			              yield true;
      // 			            }
      // 			            yield;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            switch (true) {
      // 			            case 'abc':
      // 			              yield;
      // 			            }
      // 			            yield true;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            for (const i of abc) {
      // 			              yield true;
      // 			            }
      // 			            yield;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            for (const a in b) {
      // 			              yield true;
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            for (let i=0; i<n; i+=1) {
      // 			              yield true;
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            while(true) {
      // 			              yield true
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            do {
      // 			              yield true
      // 			            }
      // 			            while(true)
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            if (true) {
      // 			              yield true;
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            var a = {};
      // 			            with (a) {
      // 			              yield true;
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            if (true) {
      // 			              yield;
      // 			            } else {
      // 			              yield true;
      // 			            }
      // 			            yield;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            if (false) {
      // 			              return;
      // 			            }
      // 			            return yield true;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            [yield true];
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            const [a = yield true] = [];
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            a || (yield true);
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            (r = yield true);
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            a + (yield true);
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            a, yield true;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            +(yield);
      // 			            [...yield];
      // 			            [...+(yield true)];
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            someLabel: {
      // 			              yield true;
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            var obj = {
      // 			              [someKey]: 'val',
      // 			              anotherKey: yield true
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            var obj = {
      // 			              [yield true]: 'val',
      // 			            }
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            `abc${yield true}`;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            tagTemp`abc${yield true}`;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            a.b[yield true].c;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            abc?.[yield true].d?.e(yield true);
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            const [a = yield true] = arr;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            const {a = yield true} = obj;
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
      //   (
      //       "
      // 			          /**
      // 			           *
      // 			           */
      // 			          function * quux () {
      // 			            import(yield true);
      // 			          }
      // 			      ",
      //       None,
      //       None,
      //   ),
    ];

    Tester::new(RequireYields::NAME, pass, fail).test();
    // Tester::new(RequireYields::NAME, pass, fail).test_and_snapshot();
}
