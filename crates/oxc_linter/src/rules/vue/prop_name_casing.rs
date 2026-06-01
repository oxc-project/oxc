use std::ops::Deref;

use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpression, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
        ObjectPropertyKind, PropertyKey, TSSignature, TSType, TSTypeName, TSTypeReference,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    ast_util::get_declaration_from_reference_id,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::Rule,
    utils::{find_property, is_vue_component_options_object_excluding_instance},
};

fn prop_name_casing_diagnostic(span: Span, name: &str, case_type: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prop \"{name}\" is not in {case_type}.")).with_label(span)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum CaseType {
    #[default]
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "snake_case")]
    SnakeCase,
}

impl CaseType {
    fn as_str(self) -> &'static str {
        match self {
            CaseType::CamelCase => "camelCase",
            CaseType::SnakeCase => "snake_case",
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PropNameCasing(Box<Config>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Config {
    case_type: CaseType,
    ignore_props: Vec<String>,
}

impl Deref for PropNameCasing {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a specific casing (camelCase or snake_case) for Vue component
    /// prop names.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent prop name casing makes templates harder to read and grep
    /// for. Pinning props to a single casing across the codebase keeps the
    /// declaration site and the call site aligned.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (default `camelCase`):
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     greeting_text: String,
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule (default `camelCase`):
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     greetingText: String,
    ///   }
    /// }
    /// </script>
    /// ```
    PropNameCasing,
    vue,
    style,
    config = PropNameCasing,
    version = "next",
);

impl Rule for PropNameCasing {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let case_type = value
            .get(0)
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "camelCase" => Some(CaseType::CamelCase),
                "snake_case" => Some(CaseType::SnakeCase),
                _ => None,
            })
            .unwrap_or_default();
        let ignore_props = value
            .get(1)
            .and_then(|v| v.get("ignoreProps"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(ToString::to_string)).collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(Self(Box::new(Config { case_type, ignore_props })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj) => {
                if !is_vue_component_options_object_excluding_instance(node, ctx) {
                    return;
                }
                let Some(props_prop) = find_property(obj, "props") else { return };
                self.check_props_value(&props_prop.value, ctx);
            }
            AstKind::CallExpression(call) => {
                if ctx.frameworks_options() != FrameworkOptions::VueSetup {
                    return;
                }
                self.check_define_props(call, ctx);
            }
            _ => {}
        }
    }
}

impl PropNameCasing {
    fn check_define_props<'a>(&self, call: &CallExpression<'a>, ctx: &LintContext<'a>) {
        let Some(ident) = call.callee.get_identifier_reference() else { return };
        if ident.name != "defineProps" {
            return;
        }
        if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
            self.check_props_value(arg, ctx);
            return;
        }
        // `defineProps<T>()` — only the same-file `interface Props { ... }` shape is checked.
        // The `import { Props } from './x'` shape requires cross-file TS type resolution and
        // is handled by tsgolint (see the test() comment for details).
        let Some(type_arguments) = call.type_arguments.as_deref() else { return };
        let Some(first_type) = type_arguments.params.first() else { return };
        self.check_type_argument(first_type, ctx);
    }

    fn check_props_value<'a>(&self, expr: &Expression<'a>, ctx: &LintContext<'a>) {
        match expr.get_inner_expression() {
            Expression::ArrayExpression(arr) => self.check_array_props(arr, ctx),
            Expression::ObjectExpression(obj) => self.check_object_props(obj, ctx),
            _ => {}
        }
    }

    fn check_array_props<'a>(&self, arr: &ArrayExpression<'a>, ctx: &LintContext<'a>) {
        for element in &arr.elements {
            let ArrayExpressionElement::StringLiteral(lit) = element else { continue };
            self.report_if_invalid(lit.value.as_str(), lit.span, ctx);
        }
    }

    fn check_object_props<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        for prop in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else { continue };
            let Some((name, span)) = property_key_static_name(&prop.key) else { continue };
            self.report_if_invalid(name.as_ref(), span, ctx);
        }
    }

    fn check_type_argument<'a>(&self, ts_type: &TSType<'a>, ctx: &LintContext<'a>) {
        match ts_type {
            TSType::TSTypeReference(type_ref) => self.check_type_reference(type_ref, ctx),
            TSType::TSTypeLiteral(type_literal) => {
                self.check_signatures(&type_literal.members, ctx);
            }
            _ => {}
        }
    }

    fn check_type_reference<'a>(&self, type_ref: &TSTypeReference<'a>, ctx: &LintContext<'a>) {
        let TSTypeName::IdentifierReference(ident_ref) = &type_ref.type_name else { return };
        let reference = ctx.scoping().get_reference(ident_ref.reference_id());
        if !reference.is_type() {
            return;
        }
        let Some(declaration) =
            get_declaration_from_reference_id(ident_ref.reference_id(), ctx.semantic())
        else {
            return;
        };
        let AstKind::TSInterfaceDeclaration(interface_decl) = declaration.kind() else { return };
        self.check_signatures(&interface_decl.body.body, ctx);
    }

    fn check_signatures<'a>(&self, signatures: &[TSSignature<'a>], ctx: &LintContext<'a>) {
        for signature in signatures {
            let (key_opt, span) = match signature {
                TSSignature::TSPropertySignature(sig) => (sig.key.static_name(), sig.key.span()),
                TSSignature::TSMethodSignature(sig) => (sig.key.static_name(), sig.key.span()),
                _ => continue,
            };
            let Some(name) = key_opt else { continue };
            self.report_if_invalid(name.as_ref(), span, ctx);
        }
    }

    fn report_if_invalid(&self, name: &str, span: Span, ctx: &LintContext<'_>) {
        if is_ignored(name, &self.ignore_props) {
            return;
        }
        if check_case(name, self.case_type) {
            return;
        }
        ctx.diagnostic(prop_name_casing_diagnostic(span, name, self.case_type.as_str()));
    }
}

/// Returns `(static_name, span_of_key_text)` for a property key when it can be
/// resolved statically. Dynamic keys (computed identifiers, calls, binary
/// expressions, etc.) return `None`.
fn property_key_static_name<'a>(
    key: &PropertyKey<'a>,
) -> Option<(std::borrow::Cow<'a, str>, Span)> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some((ident.name.as_str().into(), ident.span)),
        PropertyKey::PrivateIdentifier(_) => None,
        key => {
            // Computed key expressions: pull the inner expression and accept
            // statically resolvable literals (string / template literal /
            // identifier in parenthesized form). Other shapes (variable,
            // call, binary expression, member, this, regex literal handled
            // specially) are treated dynamically.
            let expr = key.as_expression()?.get_inner_expression();
            match expr {
                Expression::StringLiteral(lit) => Some((lit.value.as_str().into(), lit.span)),
                Expression::TemplateLiteral(tpl)
                    if tpl.expressions.is_empty() && tpl.quasis.len() == 1 =>
                {
                    let quasi = tpl.quasis.first()?;
                    let cooked = quasi.value.cooked.as_ref()?;
                    Some((cooked.as_str().into(), tpl.span))
                }
                Expression::RegExpLiteral(regex) => {
                    Some((regex.raw.as_ref()?.as_str().into(), regex.span))
                }
                _ => None,
            }
        }
    }
}

fn check_case(s: &str, case_type: CaseType) -> bool {
    match case_type {
        CaseType::CamelCase => is_camel_case(s),
        CaseType::SnakeCase => is_snake_case(s),
    }
}

fn has_symbols(s: &str) -> bool {
    // [!"#%&'()*+,./:;<=>?@[\]^`{|}] — without " ", "$", "-" and "_"
    s.chars().any(|c| {
        matches!(
            c,
            '!' | '"'
                | '#'
                | '%'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '?'
                | '@'
                | '['
                | '\\'
                | ']'
                | '^'
                | '`'
                | '{'
                | '|'
                | '}'
        )
    })
}

fn has_upper(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_uppercase())
}

fn is_camel_case(s: &str) -> bool {
    !has_symbols(s)
        && !s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && !s.chars().any(|c| matches!(c, '-' | '_') || c.is_whitespace())
}

fn is_snake_case(s: &str) -> bool {
    if has_upper(s) || has_symbols(s) {
        return false;
    }
    if s.contains('-') || s.contains("__") || s.chars().any(char::is_whitespace) {
        return false;
    }
    true
}

/// Matches upstream `toRegExpGroupMatcher`: a pattern wrapped in slashes
/// (`/foo/`) is treated as a regular expression; everything else is a
/// literal string compare.
fn is_ignored(name: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        let bytes = pattern.as_bytes();
        if bytes.len() >= 2 && bytes[0] == b'/' && bytes[bytes.len() - 1] == b'/' {
            let inner = &pattern[1..pattern.len() - 1];
            if Regex::new(inner).is_ok_and(|re| re.is_match(name)) {
                return true;
            }
        } else if pattern == name {
            return true;
        }
    }
    false
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
                    <script>
                    export default {
                      props: ['greetingText']
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: some_props
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        ...some_props,
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: ['greetingText']
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: ['greeting_text']
                    }
                    </script>
                  ",
            Some(serde_json::json!(["snake_case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        greetingText: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                  <script>
                  export default {
                    props: {
                      greetingText: String
                    }
                  }
                  </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                  <script>
                  export default {
                    props: {
                      greeting_text: String
                    }
                  }
                  </script>
                  ",
            Some(serde_json::json!(["snake_case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        ['greetingText']: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [('greetingText')]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [`greeting${'-'}text`]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        greetingText
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [greeting_text]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [greeting.text]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        ['greeting'+'-text']: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [greeting_text()]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [this]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [['greeting-text']]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [1]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [true]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [null]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        '漢字': String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        '🍻': String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        $actionEl: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        $css: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        _item: String
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["snake_case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                  <script setup>
                  defineProps({
                    greetingText: String
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            "
                  <script setup>
                  defineProps(['greetingText'])
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            r#"
                  <script setup lang="ts">
                  interface Props {
                    greetingText: number
                  }
                  defineProps<Props>()
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            "
                    <script>
                    export default {
                      props: {
                        'ignored-pattern-test': String,
                        ignored_prop: Number,
                        validProp: Boolean
                      }
                    }
                    </script>
                  ",
            Some(
                serde_json::json!([ "camelCase", { "ignoreProps": ["ignored_prop", "/^ignored-pattern-/"] } ]),
            ),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
           // NOTE: upstream `prop-name-casing` valid test using `getTypeScriptFixtureTestOptions`
           // (`import {Props2 as Props} from './test01' ; defineProps<Props>()`) is skipped here for
           // the same reason as the matching invalid case below: cross-file TypeScript type resolution
           // is the `tsgolint` domain in oxc.
    ];

    let fail = vec![
        (
            "
                    <script>
                    export default {
                      props: {
                        greeting_text: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        greeting_text: String
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: ['greeting_text']
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        greetingText: String
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["snake_case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        'greeting-text': String
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        'greeting-text': String
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["snake_case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        'greeting_text': String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        ['greeting-text']: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        greeting_text
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        'abc-123-def': String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [('greeting-text')]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        _item: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        _itemName: String
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["snake_case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [`greeting-text`]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: {
                        [/greeting-text/]: String
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                  <script setup>
                  defineProps({
                    greeting_text: String
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            "
                  <script setup>
                  defineProps(['greeting_text'])
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            r#"
                  <script setup lang="ts">
                  interface Props {
                    greeting_text: number
                  }
                  defineProps<Props>()
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            "
                    <script>
                    export default {
                      props: {
                        notIgnored_prop: String,
                        'other-pattern': Number,
                        'pattern-valid': String
                      }
                    }
                    </script>
                  ",
            Some(
                serde_json::json!(["camelCase", { "ignoreProps": ["ignored_prop", "/^pattern-/"] }]),
            ),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script>
                    export default {
                      props: ['notIgnored_prop', 'pattern_invalid', 'validProp', 'pattern-valid']
                    }
                    </script>
                  ",
            Some(
                serde_json::json!(["camelCase", { "ignoreProps": ["ignored_prop", "/^pattern-/"] }]),
            ),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
           // NOTE: upstream `prop-name-casing` invalid test using `getTypeScriptFixtureTestOptions`
           // (`import {Props3 as Props} from './test01' ; defineProps<Props>()`) is skipped here —
           // cross-file TypeScript type resolution is the `tsgolint` domain in oxc (59 typescript/* rules
           // delegate to tsgolint via the `(tsgolint)` marker). This rule body intentionally does not
           // attempt to follow type references that cross file boundaries.
    ];

    Tester::new(PropNameCasing::NAME, PropNameCasing::PLUGIN, pass, fail).test_and_snapshot();
}
