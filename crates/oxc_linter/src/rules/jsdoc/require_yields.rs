use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::{
    config::JSDocPluginSettings,
    context::LintContext,
    rule::Rule,
    utils::{
        get_function_nearest_jsdoc_node, should_ignore_as_avoid, should_ignore_as_internal,
        should_ignore_as_private,
    },
    AstNode,
};

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
    #[serde(default = "default_exempted_by", rename = "exemptedBy")]
    exempted_by: Vec<String>,
    #[serde(default, rename = "forceRequireYields")]
    force_require_yields: bool,
    #[serde(default, rename = "withGeneratorTag")]
    with_generator_tag: bool,
}
impl Default for RequireYieldsConfig {
    fn default() -> Self {
        Self {
            exempted_by: default_exempted_by(),
            force_require_yields: false,
            with_generator_tag: false,
        }
    }
}

fn default_exempted_by() -> Vec<String> {
    vec!["inheritdoc".to_string()]
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
        let config = &self.0;

        match node.kind() {
            AstKind::Function(func)
                if func.generator && (func.is_expression() || func.is_declaration()) =>
            {
                let Some(jsdocs) = get_function_nearest_jsdoc_node(node, ctx)
                    .and_then(|node| ctx.jsdoc().get_all_by_node(node))
                else {
                    return;
                };

                let settings = &ctx.settings().jsdoc;
                if jsdocs.iter().any(|jsdoc| {
                    should_ignore_as_skip(jsdoc)
                        || should_ignore_as_avoid(jsdoc, settings, &config.exempted_by)
                        || should_ignore_as_private(jsdoc, settings)
                        || should_ignore_as_internal(jsdoc, settings)
                }) {
                    return;
                }

                let jsdoc_tags = jsdocs.iter().flat_map(JSDoc::tags).collect::<Vec<_>>();

                if config.force_require_yields && is_missing_yields_tag(&jsdoc_tags, settings) {
                    // TODO: Diagnostic w/ missing yields!
                    ctx.diagnostic(RequireYieldsDiagnostic(func.span));
                    return;
                }

                if let Some(span) = is_duplicated_yields_tag(&jsdoc_tags, settings) {
                    // TODO: Diagnostic w/ duplicate yields!
                    ctx.diagnostic(RequireYieldsDiagnostic(span));
                    return;
                }

                if config.with_generator_tag {
                    if let Some(span) =
                        is_missing_yields_tag_with_generator_tag(&jsdoc_tags, settings)
                    {
                        // TODO: Diagnostic w/ generator!
                        ctx.diagnostic(RequireYieldsDiagnostic(span));
                    }
                }
            }
            AstKind::YieldExpression(yield_expr) => {
                // With this option, no needs to check `yield` value.
                // We can perform all checks in `Function` branch instead.
                if config.force_require_yields {
                    return;
                }

                // Do not check `yield` without value
                if yield_expr.argument.is_none() {
                    return;
                }

                // Find the nearest generator function
                let mut generator_func_node = None;
                let mut current_node = node;
                while let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
                    // If syntax is valid, `yield` should be inside a generator function
                    if let AstKind::Function(func) = parent_node.kind() {
                        if func.generator && (func.is_expression() || func.is_declaration()) {
                            generator_func_node = Some((func, parent_node));
                            break;
                        }
                    }
                    current_node = parent_node;
                }
                let Some((generator_func, generator_func_node)) = generator_func_node else {
                    return;
                };

                // If no JSDoc is found, skip
                let Some(jsdocs) = get_function_nearest_jsdoc_node(generator_func_node, ctx)
                    .and_then(|node| ctx.jsdoc().get_all_by_node(node))
                else {
                    return;
                };

                let settings = &ctx.settings().jsdoc;
                if jsdocs.iter().any(|jsdoc| {
                    should_ignore_as_skip(jsdoc)
                        || should_ignore_as_avoid(jsdoc, settings, &config.exempted_by)
                        || should_ignore_as_private(jsdoc, settings)
                        || should_ignore_as_internal(jsdoc, settings)
                }) {
                    return;
                }

                let jsdoc_tags = jsdocs.iter().flat_map(JSDoc::tags).collect::<Vec<_>>();

                if is_missing_yields_tag(&jsdoc_tags, settings) {
                    // TODO: Diagnostic w/ missing yields!
                    ctx.diagnostic(RequireYieldsDiagnostic(generator_func.span));
                }
            }
            _ => {}
        }
    }
}

fn should_ignore_as_skip(jsdoc: &JSDoc) -> bool {
    let ignore_tag_names = ["abstract", "virtual", "class", "constructor", "type", "interface"]
        .iter()
        .map(|s| (*s).to_string())
        .collect::<FxHashSet<_>>();

    for tag in jsdoc.tags() {
        if ignore_tag_names.contains(tag.kind.parsed()) {
            return true;
        }
    }

    false
}

fn is_missing_yields_tag(jsdoc_tags: &Vec<&JSDocTag>, settings: &JSDocPluginSettings) -> bool {
    let resolved_yields_tag_name = settings.resolve_tag_name("yields");

    for tag in jsdoc_tags {
        if tag.kind.parsed() == resolved_yields_tag_name {
            return false;
        }
    }

    true
}

fn is_duplicated_yields_tag(
    jsdoc_tags: &Vec<&JSDocTag>,
    settings: &JSDocPluginSettings,
) -> Option<Span> {
    let resolved_yields_tag_name = settings.resolve_tag_name("yields");

    let mut yields_found = false;
    for tag in jsdoc_tags {
        if tag.kind.parsed() == resolved_yields_tag_name {
            if yields_found {
                return Some(tag.kind.span);
            }

            yields_found = true;
        }
    }

    None
}

fn is_missing_yields_tag_with_generator_tag(
    jsdoc_tags: &Vec<&JSDocTag>,
    settings: &JSDocPluginSettings,
) -> Option<Span> {
    let resolved_yields_tag_name = settings.resolve_tag_name("yields");
    let resolved_generator_tag_name = settings.resolve_tag_name("generator");

    let (mut yields_found, mut generator_found) = (None, None);
    for tag in jsdoc_tags {
        let tag_name = tag.kind.parsed();

        if tag_name == resolved_yields_tag_name {
            yields_found = Some(tag.kind.span);
        }
        if tag_name == resolved_generator_tag_name {
            generator_found = Some(tag.kind.span);
        }
    }

    if let (Some(generator_tag_span), None) = (generator_found, yields_found) {
        return Some(generator_tag_span);
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
        			           * @yields Foo.
        			           */
        			          function * quux () {
			
        			            yield foo;
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * pass(without yield, no config)
        			           */
        			          function * quux () {
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
        			          function * quux () {
        			            yield;
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
        			            bar.doSomething(function * (baz) {
        			              yield baz.corge();
        			            })
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {Array}
        			           */
        			          function * quux (bar) {
        			            yield bar.doSomething(function * (baz) {
        			              yield baz.corge();
        			            })
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @inheritdoc
        			           */
        			          function * quux (foo) {
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
        			          function * quux (foo) {
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
        			          function * quux (foo) {
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @implements
        			           */
        			          function * quux (foo) {
        			            yield;
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * pass(`@override` found, settings should be default true)
        			           * @override
        			           */
        			          function * quux (foo) {

        			            yield foo;
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
        			          function * quux (foo) {
        			            yield foo;
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {object}
        			           */
        			          function * quux () {

        			            yield {a: foo};
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function * quux () {
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {undefined}
        			           */
        			          function * quux () {
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function quux () {
        			          }
        			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function * quux () {
        			            yield undefined;
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function * quux () {
        			            yield undefined;
        			          }
        			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function * quux () {
        			            yield;
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function * quux () {
        			          }
        			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @yields {void}
        			           */
        			          function * quux () {
        			            yield;
        			          }
        			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /** @type {SpecialIterator} */
        			          function * quux () {
        			            yield 5;
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @yields {Something}
        			           */
        			          async function * quux () {
        			          }
        			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           *
        			           */
        			          async function * quux () {}
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           *
        			           */
        			          const quux = async function * () {}
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @type {MyCallback}
        			           */
        			          function * quux () {
        			            yield;
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
        			      async function * foo (a) {
        			        yield;
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
                "forceRequireYields": true,
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
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @generator
        			           */
        			      ",
            Some(serde_json::json!([
              {
                "withGeneratorTag": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * pass(`@generator`+`@yields`, with config)
        			           * @generator
        			           * @yields
        			           */
                        function*() {yield 1;}
        			      ",
            Some(serde_json::json!([
              {
                "withGeneratorTag": true,
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @yields
        			           */
        			          function * quux (foo) {

        			            const a = yield foo;
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
        			          function * quux (foo) {
        			            const a = function * bar () {
        			              yield foo;
        			            }
        			          }
        			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
      			          /**
      			           * fail(`yield` with value but no `@yields`)
      			           */
      			          function * quux (foo) { yield foo; }
      			      ",
            None,
            None,
        ),
        (
            "
      			          /**
      			           *
      			           */
      			          function * quux (foo) {
      			            someLabel: {
      			              yield foo;
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
      			          function * quux (foo) {

      			            const a = yield foo;
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
      			          function * quux (foo) {
      			            yield foo;
      			          }
      			      ",
            None,
            Some(serde_json::json!({ "settings": {
        "jsdoc": {
          "tagNamePreference": {
            "yields": "yield",
          },
        },
      } })),
        ),
        (
            "
      			          /**
      			           *
      			           */
      			          function * quux() {
      			            yield 5;
      			          }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			          /**
      			           *
      			           */
      			          function * quux() {
      			            yield;
      			          }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			          /**
      			           *
      			           */
      			          const quux = async function * () {
      			            yield;
      			          }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			           /**
      			            *
      			            */
      			           async function * quux () {
      			             yield;
      			           }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			          /**
      			           *
      			           */
      			          function * quux () {
      			            yield;
      			          }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			          /**
      			           * @function
      			           * @generator
      			           */
                        function*() {}
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			          /**
      			           * @yields {undefined}
      			           * @yields {void}
      			           */
      			          function * quux (foo) {

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
      			          function * quux (foo) {
      			            yield 'bar';
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
      			      async function * foo(a) {
      			        return;
      			      }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			      /**
      			       * @param {array} a
      			       */
      			      async function * foo(a) {
      			        yield Promise.all(a);
      			      }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			      class quux {
      			        /**
      			         *
      			         */
      			        * quux () {
      			          yield;
      			        }
      			      }
      			      ",
            Some(serde_json::json!([
              {
                "forceRequireYields": true,
              },
            ])),
            None,
        ),
        (
            "
      			      /**
      			       * @param {array} a
      			       */
      			      async function * foo(a) {
      			        yield Promise.all(a);
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
      			          function * quux () {
      			            if (true) {
      			              yield;
      			            }
      			            yield true;
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
      			          function * quux () {
      			            if (yield false) {

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
      			          function * quux () {
      			            b ? yield false : true
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
      			          function * quux () {
      			            try {
      			              yield true;
      			            } catch (err) {
      			            }
      			            yield;
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
      			          function * quux () {
      			            try {
      			            } finally {
      			              yield true;
      			            }
      			            yield;
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
      			          function * quux () {
      			            try {
      			              yield;
      			            } catch (err) {
      			            }
      			            yield true;
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
      			          function * quux () {
      			            try {
      			              something();
      			            } catch (err) {
      			              yield true;
      			            }
      			            yield;
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
      			          function * quux () {
      			            switch (true) {
      			            case 'abc':
      			              yield true;
      			            }
      			            yield;
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
      			          function * quux () {
      			            switch (true) {
      			            case 'abc':
      			              yield;
      			            }
      			            yield true;
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
      			          function * quux () {
      			            for (const i of abc) {
      			              yield true;
      			            }
      			            yield;
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
      			          function * quux () {
      			            for (const a in b) {
      			              yield true;
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
      			          function * quux () {
      			            for (let i=0; i<n; i+=1) {
      			              yield true;
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
      			          function * quux () {
      			            while(true) {
      			              yield true
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
      			          function * quux () {
      			            do {
      			              yield true
      			            }
      			            while(true)
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
      			          function * quux () {
      			            if (true) {
      			              yield true;
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
      			          function * quux () {
      			            var a = {};
      			            with (a) {
      			              yield true;
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
      			          function * quux () {
      			            if (true) {
      			              yield;
      			            } else {
      			              yield true;
      			            }
      			            yield;
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
      			          function * quux () {
      			            if (false) {
      			              return;
      			            }
      			            return yield true;
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
      			          function * quux () {
      			            [yield true];
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
      			          function * quux () {
      			            const [a = yield true] = [];
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
      			          function * quux () {
      			            a || (yield true);
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
      			          function * quux () {
      			            (r = yield true);
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
      			          function * quux () {
      			            a + (yield true);
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
      			          function * quux () {
      			            a, yield true;
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
      			          function * quux () {
      			            +(yield);
      			            [...yield];
      			            [...+(yield true)];
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
      			          function * quux () {
      			            someLabel: {
      			              yield true;
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
      			          function * quux () {
      			            var obj = {
      			              [someKey]: 'val',
      			              anotherKey: yield true
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
      			          function * quux () {
      			            var obj = {
      			              [yield true]: 'val',
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
      			          function * quux () {
      			            `abc${yield true}`;
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
      			          function * quux () {
      			            tagTemp`abc${yield true}`;
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
      			          function * quux () {
      			            a.b[yield true].c;
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
      			          function * quux () {
      			            abc?.[yield true].d?.e(yield true);
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
      			          function * quux () {
      			            const [a = yield true] = arr;
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
      			          function * quux () {
      			            const {a = yield true} = obj;
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
      			          function * quux () {
      			            import(yield true);
      			          }
      			      ",
            None,
            None,
        ),
    ];

    Tester::new(RequireYields::NAME, pass, fail).test();
    // Tester::new(RequireYields::NAME, pass, fail).test_and_snapshot();
}
