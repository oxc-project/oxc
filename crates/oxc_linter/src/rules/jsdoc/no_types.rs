use oxc_ast::{
    AstKind,
    ast::{Declaration, FunctionType, MethodDefinitionType, VariableDeclaration},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::JSDoc;
use oxc_span::Span;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_function_nearest_jsdoc_node, should_ignore_as_internal, should_ignore_as_private},
};

fn no_types_diagnostic(span: Span, tag_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Types are not permitted on `@{tag_name}`."))
        .with_help("Remove the JSDoc type annotation.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoTypes(Box<NoTypesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports types on `@param` and `@returns` tags, and on `@property` tags
    /// attached to class declarations. The aliases `@arg`, `@argument`, `@return`,
    /// and `@prop` are also checked.
    ///
    /// ### Why is this bad?
    ///
    /// In TypeScript, these annotations duplicate types in the source code and can
    /// become inconsistent with them. Keep descriptions in JSDoc and types in TypeScript.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// /** @param {number} value - The value to double. */
    /// function double(value: number): number { return value * 2; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// /** @param value - The value to double. */
    /// function double(value: number): number { return value * 2; }
    /// ```
    NoTypes,
    jsdoc,
    restriction,
    fix,
    config = NoTypesConfig,
    version = "1.86.0",
    short_description = "Disallow JSDoc types that duplicate TypeScript types.",
);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct NoTypesConfig {
    /// Select the node kinds whose JSDoc should be checked. By default, all of the
    /// listed node kinds are checked. `any` also checks virtual functions marked
    /// with `@callback`, `@function`, `@func`, or `@method`, including unattached
    /// comments. In `any` mode, other comments are checked only when attached to
    /// JavaScript functions or variables initialized with functions.
    contexts: Option<Vec<NoTypesContext>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
enum NoTypesContext {
    #[serde(rename = "any")]
    Any,
    ArrowFunctionExpression,
    FunctionDeclaration,
    FunctionExpression,
    TSDeclareFunction,
    TSMethodSignature,
    ClassDeclaration,
}

impl NoTypesContext {
    fn from_kind(kind: AstKind<'_>) -> Option<Self> {
        Some(match kind {
            AstKind::Function(function) => match function.r#type {
                FunctionType::FunctionDeclaration => Self::FunctionDeclaration,
                FunctionType::FunctionExpression => Self::FunctionExpression,
                FunctionType::TSDeclareFunction => Self::TSDeclareFunction,
                FunctionType::TSEmptyBodyFunctionExpression => return None,
            },
            AstKind::ArrowFunctionExpression(_) => Self::ArrowFunctionExpression,
            AstKind::TSMethodSignature(_) => Self::TSMethodSignature,
            AstKind::Class(class) if class.is_declaration() => Self::ClassDeclaration,
            _ => return None,
        })
    }
}

impl Rule for NoTypes {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        if ctx.comments().is_empty() {
            return;
        }

        let any_context =
            self.0.contexts.as_ref().is_some_and(|c| c.contains(&NoTypesContext::Any));
        let mut checked = FxHashSet::default();

        for node in ctx.nodes() {
            if !any_context {
                let Some(context) = NoTypesContext::from_kind(node.kind()) else { continue };
                if self.0.contexts.as_ref().is_some_and(|contexts| !contexts.contains(&context)) {
                    continue;
                }
            }

            let jsdoc_node = match node.kind() {
                AstKind::Class(class) if class.is_declaration() => {
                    let parent = ctx.nodes().parent_node(node.id());
                    if matches!(
                        parent.kind(),
                        AstKind::ExportDeclaration(_) | AstKind::ExportDefaultDeclaration(_)
                    ) {
                        Some(parent)
                    } else {
                        Some(node)
                    }
                }
                AstKind::TSMethodSignature(_) | AstKind::MethodDefinition(_) => Some(node),
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    get_function_nearest_jsdoc_node(node, ctx)
                }
                _ => continue,
            };
            let Some(jsdoc_node) = jsdoc_node else { continue };
            let Some(jsdoc) = ctx.jsdoc().get_one_by_node(ctx.nodes(), jsdoc_node) else {
                continue;
            };

            // ESLint's `any` mode checks the comment's owner, not the nested function.
            if any_context
                && !is_function_or_variable(jsdoc_node.kind())
                && !is_virtual_function(jsdoc)
            {
                continue;
            }

            // Several function initializers can share one JSDoc block.
            if checked.insert(jsdoc.span.start) {
                let is_class = matches!(node.kind(), AstKind::Class(_))
                    && (!any_context || matches!(jsdoc_node.kind(), AstKind::Class(_)));
                check_tags(jsdoc, is_class, ctx);
            }
        }

        if any_context {
            // The JSDoc finder only collects leading comments. Virtual functions
            // may also be documented by trailing or otherwise unattached blocks.
            for comment in ctx.comments() {
                if !comment.is_block() {
                    continue;
                }
                let content = comment.content_span();
                let Some(raw) = ctx.source_range(content).strip_prefix('*') else { continue };
                if !raw.starts_with(char::is_whitespace) {
                    continue;
                }
                let span = Span::new(content.start + 1, content.end);
                if checked.contains(&span.start) {
                    continue;
                }
                let jsdoc = JSDoc::new(raw, span);
                if is_virtual_function(&jsdoc) {
                    check_tags(&jsdoc, false, ctx);
                }
            }
        }
    }
}

fn is_function_or_variable(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::Function(function) => matches!(
            function.r#type,
            FunctionType::FunctionDeclaration | FunctionType::FunctionExpression
        ),
        AstKind::ArrowFunctionExpression(_) => true,
        AstKind::MethodDefinition(method) => {
            method.r#type == MethodDefinitionType::MethodDefinition
        }
        AstKind::VariableDeclaration(declaration) => has_function_initializer(declaration),
        AstKind::ExportDeclaration(export) => match &export.declaration {
            Declaration::VariableDeclaration(declaration) => has_function_initializer(declaration),
            _ => false,
        },
        _ => false,
    }
}

fn has_function_initializer(declaration: &VariableDeclaration<'_>) -> bool {
    declaration.declarations.iter().any(|declarator| {
        declarator.init.as_ref().is_some_and(|init| init.without_parentheses().is_function())
    })
}

fn is_virtual_function(jsdoc: &JSDoc<'_>) -> bool {
    jsdoc
        .tags()
        .iter()
        .any(|tag| matches!(tag.kind.parsed(), "callback" | "function" | "func" | "method"))
}

fn check_tags(jsdoc: &JSDoc<'_>, is_class: bool, ctx: &LintContext<'_>) {
    let settings = &ctx.settings().jsdoc;
    if !ctx.source_range(jsdoc.span).starts_with(char::is_whitespace)
        || should_ignore_as_internal(jsdoc, settings)
        || should_ignore_as_private(jsdoc, settings)
    {
        return;
    }

    for tag in jsdoc.tags() {
        let tag_name = tag.kind.parsed();
        if !(matches!(tag_name, "param" | "arg" | "argument" | "returns" | "return")
            || is_class && matches!(tag_name, "property" | "prop"))
        {
            continue;
        }
        let Some(type_part) = tag.r#type() else { continue };
        if type_part.raw() == "{}" {
            continue;
        }

        ctx.diagnostic_with_fix(no_types_diagnostic(type_part.span, tag_name), |fixer| {
            let prefix = ctx.source_range(Span::new(tag.kind.span.end, type_part.span.start));
            // Preserve the prefix of a continuation line.
            let start = if prefix.contains(['\n', '\r']) {
                type_part.span.start
            } else {
                tag.kind.span.end
            };
            let after_type = &ctx.source_text()[type_part.span.end as usize..];
            // Avoid turning `@param {T}name` into `@paramname`.
            let replacement = if after_type.starts_with(|c: char| !c.is_whitespace() && c != '*') {
                " "
            } else {
                ""
            };
            fixer.replace(Span::new(start, type_part.span.end), replacement)
        });
    }
}

// Ported from eslint-plugin-jsdoc at 34edd048ac9914eb0d6888b4b022d4902111bc0d:
// https://github.com/gajus/eslint-plugin-jsdoc/blob/34edd048ac9914eb0d6888b4b022d4902111bc0d/test/rules/assertions/noTypes.js
#[test]
fn test_upstream() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"
          /**
           * @param foo
           */
          function quux (foo) {

          }
      ",
            None,
        ),
        (
            r"
          /**
           * @param foo
           */
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
        (
            r"
          /**
           * @function
           * @param {number} foo
           */
      ",
            None,
        ),
        (
            r"
          /**
           * @callback
           * @param {number} foo
           */
      ",
            None,
        ),
        (
            r"
        /*** Oops that's too many asterisks by accident **/
        function a () {}
      ",
            None,
        ),
    ];
    let fix = vec![
        (
            r"
          /**
           * @param {number} foo
           */
          function quux (foo) {

          }
      ",
            r"
          /**
           * @param foo
           */
          function quux (foo) {

          }
      ",
            None,
        ),
        (
            r"
      class quux {
        /**
         * @param {number} foo
         */
        bar (foo) {

        }
      }
      ",
            r"
      class quux {
        /**
         * @param foo
         */
        bar (foo) {

        }
      }
      ",
            None,
        ),
        (
            r"
          /**
           * @param {number} foo
           */
          function quux (foo) {

          }
      ",
            r"
          /**
           * @param foo
           */
          function quux (foo) {

          }
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
        (
            r"
      class quux {
        /**
         * @param {number} foo
         */
        quux (foo) {

        }
      }
      ",
            r"
      class quux {
        /**
         * @param foo
         */
        quux (foo) {

        }
      }
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
        (
            r"
          /**
           * @function
           * @param {number} foo
           */
      ",
            r"
          /**
           * @function
           * @param foo
           */
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
        (
            r"
          /**
           * @callback
           * @param {number} foo
           */
      ",
            r"
          /**
           * @callback
           * @param foo
           */
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
        (
            r"
          /**
           * @returns {number}
           */
          function quux () {

          }
      ",
            r"
          /**
           * @returns
           */
          function quux () {

          }
      ",
            None,
        ),
        (
            r"
          /**
           * Beep
           * Boop
           *
           * @returns {number}
           */
          function quux () {

          }
      ",
            r"
          /**
           * Beep
           * Boop
           *
           * @returns
           */
          function quux () {

          }
      ",
            None,
        ),
        (
            r"
        export interface B {
          /**
           * @param {string} paramA
           */
          methodB(paramB: string): void
        }
      ",
            r"
        export interface B {
          /**
           * @param paramA
           */
          methodB(paramB: string): void
        }
      ",
            None,
        ),
        (
            r"
        /**
         * @class
         * @property {object} x
         */
        class Example {
          x: number;
        }
      ",
            r"
        /**
         * @class
         * @property x
         */
        class Example {
          x: number;
        }
      ",
            None,
        ),
        (
            r"
        /**
         * Returns a Promise...
         *
         * @param {number} ms - The number of ...
         */
        const sleep = (ms: number): Promise<unknown> => {};
      ",
            r"
        /**
         * Returns a Promise...
         *
         * @param ms - The number of ...
         */
        const sleep = (ms: number): Promise<unknown> => {};
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
        (
            r"
        /**
         * Returns a Promise...
         *
         * @param {number} ms - The number of ...
         */
        export const sleep = (ms: number): Promise<unknown> => {};
      ",
            r"
        /**
         * Returns a Promise...
         *
         * @param ms - The number of ...
         */
        export const sleep = (ms: number): Promise<unknown> => {};
      ",
            Some(serde_json::json!([{"contexts": ["any"]}])),
        ),
    ];
    let fail = fix.iter().map(|(code, _, config)| (*code, config.clone())).collect::<Vec<_>>();
    Tester::new(NoTypes::NAME, NoTypes::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_regressions() {
    use crate::tester::Tester;

    let any_context = Some(serde_json::json!([{ "contexts": ["any"] }]));
    let pass = vec![
        (r"/** @param {number} value */ export function f(value) {}", any_context.clone(), None),
        (
            r"/** @param {number} value */ export default function f(value) {}",
            any_context.clone(),
            None,
        ),
        (
            r"/** @param {number} value */ export default (value => value);",
            any_context.clone(),
            None,
        ),
        (
            r"/** @function
 * @property {number} value */ export class C {}",
            any_context.clone(),
            None,
        ),
        (
            r"class C { /** @param {number} value */ f = value => value; }",
            any_context.clone(),
            None,
        ),
        (r"const c = { /** @param {number} value */ f(value) {} };", any_context.clone(), None),
        (
            r"const c = { /** @param {number} value */ f: value => value };",
            any_context.clone(),
            None,
        ),
        (
            r"const a = 1, /** @param {number} value */ f = value => value;",
            any_context.clone(),
            None,
        ),
        (r"/** @type {number} */ const value = 1;", None, None),
        (r"/** @param {number} value */ const value = 1;", None, None),
        (r"/** @param {number} value */ const value = 1;", any_context.clone(), None),
        (r"/** @param {number} value */", any_context.clone(), None),
        (r"/** @property {number} value */ class C {}", any_context.clone(), None),
        (
            r"interface I {
/** @param {number} value */
method(value: number): void;
}",
            any_context.clone(),
            None,
        ),
        (r"/** @param {} value */ function f(value) {}", None, None),
        (r"/** @param value - See {@link Number}. */ function f(value) {}", None, None),
        (r"/** @throws {Error} When invalid. */ function f() {}", None, None),
        (
            r"/** @param {number} value */
/** Newer documentation. */
function f(value) {}",
            None,
            None,
        ),
        (
            r"/** @param {number} value */ function f(value) {}",
            Some(serde_json::json!([{"contexts": []}])),
            None,
        ),
        (
            r"/** @param {number} value */ function f(value) {}",
            Some(serde_json::json!([{"contexts": ["ClassDeclaration"]}])),
            None,
        ),
        (
            r"/** @input {number} value */ function f(value) {}",
            None,
            Some(
                serde_json::json!({"settings": {"jsdoc": {"tagNamePreference": {"param": "input"}}}}),
            ),
        ),
        (
            r"/** @param {number} value
 * @internal
 */ function f(value) {}",
            None,
            Some(serde_json::json!({"settings": {"jsdoc": {"ignoreInternal": true}}})),
        ),
        (
            r"/** @param {number} value
 * @private
 */ function f(value) {}",
            None,
            Some(serde_json::json!({"settings": {"jsdoc": {"ignorePrivate": true}}})),
        ),
        (
            r"/** @callback F
 * @param {number} value
 * @access private
 */",
            any_context.clone(),
            Some(serde_json::json!({"settings": {"jsdoc": {"ignorePrivate": true}}})),
        ),
        (r"/*** @param {number} value */ function f(value) {}", None, None),
        (
            r"/** @param {number} value */ function outer(value) { return function inner() {}; }",
            Some(serde_json::json!([{"contexts": ["FunctionExpression"]}])),
            None,
        ),
    ];
    let fail = vec![
        (
            r"/** @param {number} value */ export const a = 1, f = value => value;",
            any_context.clone(),
            None,
        ),
        (r"/** @param {number} value */ const f = value => value, g = value => value;", None, None),
        (r"class C { /** @param {number} value */ f = value => value; }", None, None),
        (r"const c = { /** @param {number} value */ f(value) {} };", None, None),
        (r"const c = { /** @param {number} value */ f: value => value };", None, None),
        (r"const a = 1, /** @param {number} value */ f = value => value;", None, None),
        (r"class C { /** @param {number} value */ f(value) {} }", any_context.clone(), None),
        (r"/** @arg {number} value */ function f(value) {}", None, None),
        (r"/** @argument {number} value */ function f(value) {}", None, None),
        (r"/** @return {number} */ function f() { return 1; }", None, None),
        (r"/** @prop {number} value */ class C {}", None, None),
        (r"/** @param { } value */ function f(value) {}", None, None),
        (r"/** @param {number} value */ declare function f(value: number): void;", None, None),
        (
            r"/** @param {number} value */ declare function f(value: number): void;",
            Some(serde_json::json!([{"contexts": ["TSDeclareFunction"]}])),
            None,
        ),
        (r"/** @param {number} value */ const f = function(value) {};", None, None),
        (
            r"/** @param {number} value */ const f = (value) => {};",
            Some(serde_json::json!([{"contexts": ["ArrowFunctionExpression"]}])),
            None,
        ),
        (r"/** @property {number} value */ export class C {}", None, None),
        (r"/** @property {number} value */ export default class C {}", None, None),
        (
            r"/** @function
 * @property {number} value */ class C {}",
            any_context.clone(),
            None,
        ),
        (
            r"/** @func f
 * @param {number} value
 */",
            any_context.clone(),
            None,
        ),
        (
            r"/** @method f
 * @returns {number}
 */",
            any_context.clone(),
            None,
        ),
        (
            r"const value = 1; /** @callback F
 * @param {number} value
 */",
            any_context.clone(),
            None,
        ),
        (
            r"/** @param {number} value */ function f(value) {}",
            None,
            Some(
                serde_json::json!({"settings": {"jsdoc": {"tagNamePreference": {"param": "input"}}}}),
            ),
        ),
    ];
    let fix = vec![
        (
            r"/** @param   {number}  value */ function f(value) {}",
            r"/** @param  value */ function f(value) {}",
            None,
        ),
        (
            r"/** @param {number}value */ function f(value) {}",
            r"/** @param value */ function f(value) {}",
            None,
        ),
        (
            r"/** @param{number}value */ function f(value) {}",
            r"/** @param value */ function f(value) {}",
            None,
        ),
        (
            r"/** @param {number} [value=1] - Description. */ function f(value = 1) {}",
            r"/** @param [value=1] - Description. */ function f(value = 1) {}",
            None,
        ),
        (
            r"/** @param {number} café */ function f(café) {}",
            r"/** @param café */ function f(café) {}",
            None,
        ),
        (
            "/**\r\n * @param {number} value\r\n * @returns {number}\r\n */\r\nfunction f(value) { return value; }",
            "/**\r\n * @param value\r\n * @returns\r\n */\r\nfunction f(value) { return value; }",
            None,
        ),
        (
            r"/**
 * @param {{
 *   value: number
 * }} options
 */
function f(options) {}",
            r"/**
 * @param options
 */
function f(options) {}",
            None,
        ),
        (
            r"/** @param {number} value */ const f = value => value, g = value => value;",
            r"/** @param value */ const f = value => value, g = value => value;",
            None,
        ),
        (
            r"const value = 1; /** @callback F
 * @param {number} value
 */",
            r"const value = 1; /** @callback F
 * @param value
 */",
            any_context,
        ),
    ];
    Tester::new(NoTypes::NAME, NoTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_snapshot_suffix("regressions")
        .test_and_snapshot();
}

#[test]
fn test_unsupported_contexts() {
    for value in [
        serde_json::json!([{ "contexts": ["FunctionDeclaration[async=true]"] }]),
        serde_json::json!([{ "contexts": [{ "context": "any", "comment": "JsdocBlock" }] }]),
    ] {
        assert!(NoTypes::from_configuration(value).is_err());
    }
}

#[test]
fn test_contexts_and_multiline_types() {
    use crate::tester::Tester;

    let any_context = Some(serde_json::json!([{ "contexts": ["any"] }]));

    let mut pass = vec![
        (r"/** @param {number} x */ const callbacks = [x => x];", any_context.clone()),
        (r"/** @param {number} x */ export const callbacks = [x => x];", any_context.clone()),
        (
            r"/** @param {number} x */ const callbacks = true ? (x => x) : (x => -x);",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ export const callbacks = true ? (x => x) : (x => -x);",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ const callbacks = (x => x) as (x: number) => number;",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ export const callbacks = (x => x) as (x: number) => number;",
            any_context.clone(),
        ),
        (
            r"abstract class C { /** @param {number} x */ abstract f(x: number): void; }",
            any_context.clone(),
        ),
        (
            r"/**
 * @param
 * x - Example {number}
 */
function f(x) {}",
            None,
        ),
        (
            r"/**
 * @returns
 * See {@link Number}
 */
function f() {}",
            None,
        ),
        (r"/** @param * {number} x */ function f(x) {}", None),
        (
            r"/**
 * @param
 * {} x
 */
function f(x) {}",
            None,
        ),
    ];
    for code in [
        r"declare class C { /** @param {number} x */ f(x: number): void; }",
        r"abstract class C { /** @param {number} x */ abstract f(x: number): void; }",
        r"class C { /** @param {number} x */ f(x: number): number; f(x: number) { return x; } }",
    ] {
        for config in [
            None,
            Some(serde_json::json!([{ "contexts": ["FunctionExpression"] }])),
            Some(serde_json::json!([{ "contexts": ["TSDeclareFunction"] }])),
        ] {
            pass.push((code, config));
        }
    }

    let fix = vec![
        (
            r"declare class C { /** @param {number} x */ f(x: number): void; }",
            r"declare class C { /** @param x */ f(x: number): void; }",
            any_context.clone(),
        ),
        (
            r"class C { /** @param {number} x */ f(x: number): number; f(x: number) { return x; } }",
            r"class C { /** @param x */ f(x: number): number; f(x: number) { return x; } }",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ const f = ((x => x));",
            r"/** @param x */ const f = ((x => x));",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ export const f = (function(x) {});",
            r"/** @param x */ export const f = (function(x) {});",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ const callbacks = [x => x], f = x => x;",
            r"/** @param x */ const callbacks = [x => x], f = x => x;",
            any_context.clone(),
        ),
        (
            r"/** @param {number} x */ export const callbacks = [x => x], f = function(x) {};",
            r"/** @param x */ export const callbacks = [x => x], f = function(x) {};",
            any_context.clone(),
        ),
        (
            r"/** @function
 * @param {number} x */ const callbacks = [x => x];",
            r"/** @function
 * @param x */ const callbacks = [x => x];",
            any_context,
        ),
        (
            r"/**
 * @param
 * {number} x
 */
function f(x) {}",
            r"/**
 * @param
 *  x
 */
function f(x) {}",
            None,
        ),
        (
            r"/**
 * @returns
 * {number} The result.
 */
function f() {return 1;}",
            r"/**
 * @returns
 *  The result.
 */
function f() {return 1;}",
            None,
        ),
        (
            r"/**
 * @property
 * {number} x
 */
class C {}",
            r"/**
 * @property
 *  x
 */
class C {}",
            None,
        ),
        (
            "/**\r\n * @param\r\n * {number} x\r\n */\r\nfunction f(x) {}",
            "/**\r\n * @param\r\n *  x\r\n */\r\nfunction f(x) {}",
            None,
        ),
        (
            r"/**
 * @param
 *
 * {number} x
 */
function f(x) {}",
            r"/**
 * @param
 *
 *  x
 */
function f(x) {}",
            None,
        ),
        (
            r"/**
 * @param
 {number} x
 */
function f(x) {}",
            r"/**
 * @param
  x
 */
function f(x) {}",
            None,
        ),
    ];
    let fail = fix.iter().map(|(code, _, config)| (*code, config.clone())).collect::<Vec<_>>();
    Tester::new(NoTypes::NAME, NoTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_snapshot_suffix("contexts_and_multiline_types")
        .test_and_snapshot();
}
