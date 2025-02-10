use std::sync::Mutex;

use lazy_static::lazy_static;
use oxc_ast::{ast::MethodDefinitionKind, AstKind};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, JSDoc};
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_params, default_true, get_function_nearest_jsdoc_node, should_ignore_as_avoid,
        should_ignore_as_internal, should_ignore_as_private, ParamKind,
    },
};

#[derive(Debug, Default, Clone)]
pub struct RequireParam(Box<RequireParamConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all function parameters are documented with JSDoc `@param` tags.
    ///
    /// ### Why is this bad?
    ///
    /// The rule is aimed at enforcing code quality and maintainability by requiring that all function parameters are documented.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @param foo */
    /// function quux (foo, bar) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @param foo */
    /// function quux (foo) {}
    /// ```
    RequireParam,
    jsdoc,
    pedantic,
);

#[derive(Debug, Clone, Deserialize)]
struct RequireParamConfig {
    #[serde(default = "default_exempted_by", rename = "exemptedBy")]
    exempted_by: Vec<String>,
    #[serde(default = "default_true", rename = "checkConstructors")]
    check_constructors: bool,
    #[serde(default, rename = "checkGetters")]
    check_getters: bool,
    #[serde(default, rename = "checkSetters")]
    check_setters: bool,
    #[serde(default = "default_true", rename = "checkDestructuredRoots")]
    check_destructured_roots: bool,
    #[serde(default = "default_true", rename = "checkDestructured")]
    check_destructured: bool,
    #[serde(default, rename = "checkRestProperty")]
    check_rest_property: bool,
    #[serde(default = "default_check_types_pattern", rename = "checkTypesPattern")]
    check_types_pattern: String,
    // TODO: Support this config
    // #[serde(default, rename = "useDefaultObjectProperties")]
    // use_default_object_properties: bool,
}
impl Default for RequireParamConfig {
    fn default() -> Self {
        Self {
            exempted_by: default_exempted_by(),
            check_constructors: false,
            check_getters: default_true(),
            check_setters: default_true(),
            check_destructured_roots: default_true(),
            check_destructured: default_true(),
            check_rest_property: false,
            check_types_pattern: default_check_types_pattern(),
        }
    }
}
fn default_exempted_by() -> Vec<String> {
    vec!["inheritdoc".to_string()]
}
fn default_check_types_pattern() -> String {
    "^(?:[oO]bject|[aA]rray|PlainObject|Generic(?:Object|Array))$".to_string() // spellchecker:disable-line
}

// For perf, cache regex is needed
lazy_static! {
    static ref REGEX_CACHE: Mutex<FxHashMap<String, Regex>> = Mutex::new(FxHashMap::default());
}

impl Rule for RequireParam {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Collected targets from `FormalParameters`
        let params_to_check = match node.kind() {
            AstKind::Function(func) if !func.is_typescript_syntax() => collect_params(&func.params),
            AstKind::ArrowFunctionExpression(arrow_func) => collect_params(&arrow_func.params),
            // If not a function, skip
            _ => return,
        };

        let Some(func_def_node) = get_function_nearest_jsdoc_node(node, ctx) else {
            return;
        };
        // If no JSDoc is found, skip
        let Some(jsdocs) = ctx.jsdoc().get_all_by_node(func_def_node) else {
            return;
        };

        let config = &self.0;
        let settings = &ctx.settings().jsdoc;

        // If config disabled checking, skip
        if let AstKind::MethodDefinition(method_def) = func_def_node.kind() {
            match method_def.kind {
                MethodDefinitionKind::Get => {
                    if !config.check_getters {
                        return;
                    }
                }
                MethodDefinitionKind::Set => {
                    if !config.check_setters {
                        return;
                    }
                }
                MethodDefinitionKind::Constructor => {
                    if !config.check_constructors {
                        return;
                    }
                }
                MethodDefinitionKind::Method => {}
            }
        }

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

        // Collected JSDoc `@param` tags
        let tags_to_check = collect_tags(&jsdocs, settings.resolve_tag_name("param"));
        let shallow_tags =
            tags_to_check.iter().filter(|(name, _)| !name.contains('.')).collect::<Vec<_>>();

        let mut regex_cache = REGEX_CACHE.lock().unwrap();
        let check_types_regex =
            regex_cache.entry(config.check_types_pattern.clone()).or_insert_with(|| {
                Regex::new(config.check_types_pattern.as_str())
                    .expect("`config.checkTypesPattern` should be a valid regex pattern")
            });

        let mut violations = vec![];
        for (idx, param) in params_to_check.iter().enumerate() {
            match param {
                ParamKind::Single(param) => {
                    if !config.check_rest_property && param.is_rest {
                        continue;
                    }

                    if !tags_to_check.iter().any(|(name, _)| **name == param.name) {
                        violations.push(param.span);
                    }
                }
                ParamKind::Nested(params) => {
                    // If false, skip nested root
                    if !config.check_destructured_roots {
                        continue;
                    }

                    let matched_param_tag = shallow_tags.get(idx);

                    // If {type} exists...
                    if let Some((_, Some(r#type))) = matched_param_tag {
                        // ... and doesn't match the pattern, skip
                        if !check_types_regex.is_match(r#type) {
                            continue;
                        }
                    }

                    // If false, skip nested props
                    if !config.check_destructured {
                        continue;
                    }

                    let root_name = matched_param_tag.map_or("", |(name, _)| name);
                    let mut not_checking_names = FxHashSet::default();
                    for param in params {
                        if !config.check_rest_property && param.is_rest {
                            continue;
                        }

                        let full_param_name = format!("{root_name}.{}", param.name);
                        for (name, type_part) in &tags_to_check {
                            if !is_name_equal(name, &full_param_name) {
                                continue;
                            }
                            let Some(r#type) = type_part else {
                                continue;
                            };
                            if check_types_regex.is_match(r#type) {
                                continue;
                            }

                            not_checking_names.insert(name);
                        }

                        if not_checking_names.iter().any(|&name| full_param_name.starts_with(name))
                        {
                            continue;
                        }

                        if !tags_to_check
                            .iter()
                            .any(|(name, _)| is_name_equal(name, &full_param_name))
                        {
                            violations.push(param.span);
                        }
                    }
                }
            }
        }

        if !violations.is_empty() {
            let labels = violations
                .iter()
                .map(|span| LabeledSpan::new_with_span(None, *span))
                .collect::<Vec<_>>();
            ctx.diagnostic(
                OxcDiagnostic::warn("Missing JSDoc `@param` declaration for function parameters.")
                    .with_help("Add `@param` tag with name.")
                    .with_labels(labels),
            );
        }
    }
}

fn collect_tags<'a>(
    jsdocs: &[JSDoc<'a>],
    resolved_param_tag_name: &str,
) -> Vec<(&'a str, Option<&'a str>)> {
    let mut collected = vec![];

    for tag in jsdocs
        .iter()
        .flat_map(JSDoc::tags)
        .filter(|tag| tag.kind.parsed() == resolved_param_tag_name)
    {
        let (type_part, Some(name_part), _) = tag.type_name_comment() else {
            continue;
        };

        let name = name_part.parsed();
        // thisParam is special, not collected as `FormalParameter`, should be ignored
        if name == "this" {
            continue;
        }

        collected.push((name, type_part.map(|p| p.parsed())));
    }

    collected
}

fn should_ignore_as_custom_skip(jsdoc: &JSDoc) -> bool {
    jsdoc.tags().iter().any(|tag| "type" == tag.kind.parsed())
}

/// Compare to string param names without quotes
/// e.g. `foo."bar"`
fn is_name_equal(a: &str, b: &str) -> bool {
    let mut a_chars = a.chars().filter(|&c| c != '"');
    let mut b_chars = b.chars().filter(|&c| c != '"');

    loop {
        match (a_chars.next(), b_chars.next()) {
            (Some(ac), Some(bc)) if ac == bc => continue,
            (None, None) => return true, // Both done
            _ => return false,           // Either one is done, or not equal
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("
			          /**
			           * @param foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @param root0
			           * @param root0.foo
			           */
			          function quux ({foo}) {
			
			          }
			      ", None, None),
("
			          /**
			           * @param root0
			           * @param root0.foo
			           * @param root1
			           * @param root1.bar
			           */
			          function quux ({foo}, {bar}) {
			
			          }
			      ", None, None),
("
			          /**
			           * @param arg0
			           * @param arg0.foo
			           * @param arg1
			           * @param arg1.bar
			           */
			          function quux ({foo}, {bar}) {
			
			          }
			      ", Some(serde_json::json!([        {          "unnamedRootBase": [            "arg",          ],        },      ])), None),
("
			          /**
			           * @param arg
			           * @param arg.foo
			           * @param config0
			           * @param config0.bar
			           * @param config1
			           * @param config1.baz
			           */
			          function quux ({foo}, {bar}, {baz}) {
			
			          }
			      ", Some(serde_json::json!([        {          "unnamedRootBase": [            "arg", "config",          ],        },      ])), None),
("
			          /**
			           * @inheritdoc
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @arg foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "tagNamePreference": {            "param": "arg",          },        },      } }))),
("
			          /**
			           * @override
			           * @param foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @override
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @override
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "overrideReplacesDocs": true,        },      } }))),
("
			          /**
			           * @ignore
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "ignoreReplacesDocs": true,        },      } }))),
("
			          /**
			           * @implements
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "implementsReplacesDocs": true,        },      } }))),
("
			          /**
			           * @implements
			           * @param foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @augments
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "augmentsExtendsReplacesDocs": true,        },      } }))),
("
			          /**
			           * @augments
			           * @param foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @extends
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "augmentsExtendsReplacesDocs": true,        },      } }))),
("
			          /**
			           * @extends
			           * @param foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @internal
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "ignoreInternal": true,        },      } }))),
("
			          /**
			           * @private
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "ignorePrivate": true,        },      } }))),
("
			          /**
			           * @access private
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({ "settings": {        "jsdoc": {          "ignorePrivate": true,        },      } }))),
("
			          // issue 182: optional chaining
			          /** @const {boolean} test */
			          const test = something?.find(_ => _)
			      ", None, None), // {        "parser": babelEslintParser,      },
("
			          /**
			           * @type {MyCallback}
			           */
			          function quux () {
			
			          }
			      ", Some(serde_json::json!([        {          "exemptedBy": [            "type",          ],        },      ])), None),
("
			        export class SomeClass {
			          /**
			           * @param property
			           */
			          constructor(private property: string) {}
			        }
			      ", None, None), // {        "parser": typescriptEslintParser,        "sourceType": "module",      },
("
			      /**
			       * Assign the project to an employee.
			       *
			       * @param {object} employee - The employee who is responsible for the project.
			       * @param {string} employee.name - The name of the employee.
			       * @param {string} employee.department - The employee's department.
			       */
			      function assign({name, department}) {
			        // ...
			      }
			      ", None, None),
("
			    export abstract class StephanPlugin<O, D> {
			
			        /**
			         * Called right after Stephan loads the plugin file.
			         *
			         * @example
			         *```typescript
			         * type Options = {
			         *      verbose?: boolean;
			         *      token?: string;
			         * }
			         * ```
			         *
			         * Note that your Options type should only have optional properties...
			         *
			         * @param args Arguments compiled and provided by StephanClient.
			         * @param args.options The options as provided by the user, or an empty object if not provided.
			         * @param args.client The options as provided by the user, or an empty object if not provided.
			         * @param defaultOptions The default options as provided by the plugin, or an empty object.
			         */
			        public constructor({options, client}: {
			            options: O;
			            client: unknown;
			        }, defaultOptions: D) {
			
			        }
			    }
			      ", None, None), // {        "parser": typescriptEslintParser      },
("
			        export abstract class StephanPlugin<O, D> {
			
			        /**
			         * Called right after Stephan loads the plugin file.
			         *
			         * @example
			         *```typescript
			         * type Options = {
			         *      verbose?: boolean;
			         *      token?: string;
			         * }
			         * ```
			         *
			         * Note that your Options type should only have optional properties...
			         *
			         * @param args Arguments compiled and provided by StephanClient.
			         * @param args.options The options as provided by the user, or an empty object if not provided.
			         * @param args.client The options as provided by the user, or an empty object if not provided.
			         * @param args.client.name The name of the client.
			         * @param defaultOptions The default options as provided by the plugin, or an empty object.
			         */
			        public constructor({ options, client: { name } }: {
			            options: O;
			            client: { name: string };
			        }, defaultOptions: D) {
			
			        }
			    }
			      ", None, None), // {        "parser": typescriptEslintParser      },
("
			      /**
			      * @param {string} cb
			      */
			      function createGetter (cb) {
			        return function (...args) {
			          cb();
			        };
			      }
			      ", None, None),
("
			      /**
			       * @param cfg
			       * @param cfg.num
			       */
			      function quux ({num, ...extra}) {
			      }
			      ", None, None),
("
			      /**
			      * Converts an SVGRect into an object.
			      * @param {SVGRect} bbox - a SVGRect
			      */
			      const bboxToObj = function ({x, y, width, height}) {
			        return {x, y, width, height};
			      };
			      ", None, None),
("
			      /**
			      * Converts an SVGRect into an object.
			      * @param {object} bbox - a SVGRect
			      */
			      const bboxToObj = function ({x, y, width, height}) {
			        return {x, y, width, height};
			      };
			      ", Some(serde_json::json!([        {          "checkTypesPattern": "SVGRect",        },      ])), None),
("
			      class CSS {
			        /**
			         * Set one or more CSS properties for the set of matched elements.
			         *
			         * @param {Object} propertyObject - An object of property-value pairs to set.
			         */
			        setCssObject(propertyObject: {[key: string]: string | number}): void {
			        }
			      }
			      ", None, None), // {        "parser": typescriptEslintParser      },
("
			          /**
			           * @param foo
			           * @param bar
			           * @param cfg
			           */
			          function quux (foo, bar, {baz}) {
			
			          }
			      ", Some(serde_json::json!([        {          "checkDestructured": false,        },      ])), None),
("
			          /**
			           * @param foo
			           * @param bar
			           */
			          function quux (foo, bar, {baz}) {
			
			          }
			      ", Some(serde_json::json!([        {          "checkDestructuredRoots": false,        },      ])), None),
(r#"
			          /**
			           * @param root
			           * @param root.foo
			           */
			          function quux ({"foo": bar}) {
			
			          }
			      "#, None, None),
(r#"
			          /**
			           * @param root
			           * @param root."foo"
			           */
			          function quux ({foo: bar}) {
			
			          }
			      "#, None, None),
("
			      /**
			       * Description.
			       * @param {string} b Description `/**`.
			       */
			      module.exports = function a(b) {
			        console.info(b);
			      };
			      ", None, None),
("
			      /**
			       * Description.
			       * @param {Object} options Options.
			       * @param {FooBar} options.foo foo description.
			       */
			      function quux ({ foo: { bar } }) {}
			      ", None, None),
("
			      /**
			       * Description.
			       * @param {FooBar} options
			       * @param {Object} options.foo
			       */
			      function quux ({ foo: { bar } }) {}
			      ", Some(serde_json::json!([        {          "checkTypesPattern": "FooBar",        },      ])), None),
(r#"
			      /**
			       * @param obj
			       * @param obj.data
			       * @param obj.data."0"
			       * @param obj.data."1"
			       * @param obj.data."2"
			       * @param obj.defaulting
			       * @param obj.defaulting."0"
			       * @param obj.defaulting."1"
			       */
			      function Item({
			        data: [foo, bar, ...baz],
			        defaulting: [quux, xyz] = []
			      }) {
			      }
			      "#, None, None),
// ("
// 			      /**
// 			      * Returns a number.
// 			      * @param {Object} props Props.
// 			      * @param {Object} props.prop Prop.
// 			      * @return {number} A number.
// 			      */
// 			      export function testFn1 ({ prop = { a: 1, b: 2 } }) {
// 			      }
// 			      ", Some(serde_json::json!([        {          "useDefaultObjectProperties": false,        },      ])), None), // {        "sourceType": "module",      },
("
			      /**
			       * @param this The this object
			       * @param bar number to return
			       * @returns number returned back to caller
			       */
			      function foo(this: T, bar: number): number {
			        console.log(this.name);
			        return bar;
			      }
			      ", None, None), // {        "parser": typescriptEslintParser      },
("
			      /**
			       * @param bar number to return
			       * @returns number returned back to caller
			       */
			      function foo(this: T, bar: number): number {
			        console.log(this.name);
			        return bar;
			      }
			      ", None, None), // {        "parser": typescriptEslintParser      },
("
			        /**
			         * Returns the sum of two numbers
			         * @param options Object to destructure
			         * @param options.a First value
			         * @param options.b Second value
			         * @returns Sum of a and b
			         */
			        function sumDestructure(this: unknown, { a, b }: { a: number, b: number }) {
			          return a + b;
			        }
			      ", None, None), // {        "parser": typescriptEslintParser,      }
    ];

    let fail = vec![
        (
            "
			          /**
			           *
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
			           *
			           */
			          function quux ({foo}) {
			
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
			          function quux (foo, bar, {baz}) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "checkDestructured": false,        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           * @param foo
			           */
			          function quux (foo, bar, {baz}) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "checkDestructuredRoots": false,        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux ({foo}) {
			
			          }
			      ",
            Some(serde_json::json!([        {          "enableFixer": false,        },      ])),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux ({foo: bar = 5} = {}) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param
			           */
			          function quux ({foo}) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param
			           */
			          function quux ({foo}) {
			
			          }
			      ",
            Some(serde_json::json!([        {          "autoIncrementBase": 1,        },      ])),
            None,
        ),
        (
            "
			          /**
			           * @param options
			           */
			          function quux ({foo}) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param
			           */
			          function quux ({ foo, bar: { baz }}) {
			
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
			          function quux ({foo}, {bar}) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "unnamedRootBase": [            "arg",          ],        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux ({foo}, {bar}) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "unnamedRootBase": [            "arg", "config",          ],        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux ({foo}, {bar}) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "enableRootFixer": false,          "unnamedRootBase": [            "arg", "config",          ],        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           *
			           */
			          function quux (foo, bar) {
			
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
			          function quux (foo, bar) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param bar
			           */
			          function quux (foo, bar, baz) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param foo
			           * @param bar
			           */
			          function quux (foo, bar, baz) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param baz
			           */
			          function quux (foo, bar, baz) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param
			           */
			          function quux (foo) {
			
			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "tagNamePreference": {            "param": "arg",          },        },      } }),
            ),
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
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "overrideReplacesDocs": false,        },      } }),
            ),
        ),
        (
            "
			          /**
			           * @ignore
			           */
			          function quux (foo) {
			
			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "ignoreReplacesDocs": false,        },      } }),
            ),
        ),
        (
            "
			          /**
			           * @implements
			           */
			          function quux (foo) {
			
			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "implementsReplacesDocs": false,        },      } }),
            ),
        ),
        (
            "
			          /**
			           * @augments
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
			           * @extends
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
			       *
			       */
			      function quux ({bar, baz}, foo) {
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
			      function quux (foo, {bar, baz}) {
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
			      function quux ([bar, baz], foo) {
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
			          }
			      ",
            Some(
                serde_json::json!([        {          "exemptedBy": [            "notPresent",          ],        },      ]),
            ),
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
            Some(serde_json::json!([        {          "exemptedBy": [],        },      ])),
            None,
        ),
        (
            "
			          /**
			           * Assign the project to a list of employees.
			           * @param {object[]} employees - The employees who are responsible for the project.
			           * @param {string} employees[].name - The name of an employee.
			           * @param {string} employees[].department - The employee's department.
			           */
			          function assign (employees, name) {
			
			          };
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param baz
			           * @param options
			           */
			          function quux (baz, {foo: bar}) {
			
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
			
			          }
			      ",
            Some(serde_json::json!([        {          "enableFixer": false,        },      ])),
            None,
        ),
        (
            "
			      class Client {
			        /**
			         * Set collection data.
			         * @return {Promise<Object, Error>}
			         */
			        async setData(
			          data: { last_modified?: number }
			        ) {}
			      }
			      ",
            None,
            None,
        ), // {        "parser": typescriptEslintParser      },
        (
            "
			      /**
			       * @param cfg
			       * @param cfg.num
			       */
			      function quux ({num, ...extra}) {
			      }
			      ",
            Some(
                serde_json::json!([        {          "checkRestProperty": true,        },      ]),
            ),
            None,
        ),
        (
            "
			      /**
			       * @param cfg
			       * @param cfg.opts
			       * @param cfg.opts.num
			       */
			      function quux ({opts: {num, ...extra}}) {
			      }
			      ",
            Some(
                serde_json::json!([        {          "checkRestProperty": true,        },      ]),
            ),
            None,
        ),
        (
            r#"
			      /**
			       * @param {GenericArray} cfg
			       * @param {number} cfg."0"
			       */
			      function baar ([a, ...extra]) {
			        //
			      }
			      "#,
            Some(
                serde_json::json!([        {          "checkRestProperty": true,        },      ]),
            ),
            None,
        ),
        (
            "
			      /**
			       * @param a
			       */
			      function baar (a, ...extra) {
			        //
			      }
			      ",
            Some(
                serde_json::json!([        {          "checkRestProperty": true,        },      ]),
            ),
            None,
        ),
        (
            "
			      /**
			       * Converts an SVGRect into an object.
			       * @param {SVGRect} bbox - a SVGRect
			       */
			      const bboxToObj = function ({x, y, width, height}) {
			        return {x, y, width, height};
			      };
			      ",
            Some(
                serde_json::json!([        {          "checkTypesPattern": "SVGRect",        },      ]),
            ),
            None,
        ),
        (
            "
			      /**
			       * Converts an SVGRect into an object.
			       * @param {object} bbox - a SVGRect
			       */
			      const bboxToObj = function ({x, y, width, height}) {
			        return {x, y, width, height};
			      };
			      ",
            None,
            None,
        ),
        (
            "
			      module.exports = class GraphQL {
			        /**
			         * @param fetchOptions
			         * @param cacheKey
			         */
			        fetch = ({ url, ...options }, cacheKey) => {
			        }
			      };
			      ",
            Some(
                serde_json::json!([        {          "checkRestProperty": true,        },      ]),
            ),
            None,
        ), // {        "parser": babelEslintParser,      },
        (
            "
			(function() {
				/**
				 * A function.
				 */
				function f(param) {
					return !param;
				}
			})();
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Description.
			       * @param {Object} options
			       * @param {Object} options.foo
			       */
			      function quux ({ foo: { bar } }) {}
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Description.
			       * @param {FooBar} options
			       * @param {FooBar} options.foo
			       */
			      function quux ({ foo: { bar } }) {}
			      ",
            Some(
                serde_json::json!([        {          "checkTypesPattern": "FooBar",        },      ]),
            ),
            None,
        ),
        (
            "
			      /**
			       * Description.
			       * @param {Object} options
			       * @param {FooBar} foo
			       */
			      function quux ({ foo: { bar } }) {}
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Description.
			       * @param {Object} options
			       * @param options.foo
			       */
			      function quux ({ foo: { bar } }) {}
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Description.
			       * @param {object} options Options.
			       * @param {object} options.foo A description.
			       * @param {object} options.foo.bar
			       */
			      function foo({ foo: { bar: { baz } }}) {}
			      ",
            None,
            None,
        ),
        // (
        //     "
        // 			      /**
        // 			      * Returns a number.
        // 			      * @param {Object} props Props.
        // 			      * @param {Object} props.prop Prop.
        // 			      * @return {number} A number.
        // 			      */
        // 			      export function testFn1 ({ prop = { a: 1, b: 2 } }) {
        // 			      }
        // 			      ",
        //     Some(
        //         serde_json::json!([        {          "useDefaultObjectProperties": true,        },      ]),
        //     ),
        //     None,
        // ), // {        "sourceType": "module",      },
        (
            "
			        /** Foo. */
			        function foo(a, b, c) {}
			      ",
            None,
            None,
        ),
    ];

    Tester::new(RequireParam::NAME, RequireParam::PLUGIN, pass, fail).test_and_snapshot();
}
