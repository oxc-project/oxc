use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

#[cfg(test)]
mod tests;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        parse_jest_fn_call, JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, PossibleJestNode,
    },
};

fn prefer_lowercase_title_diagnostic(title: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce lowercase test names")
        .with_help(format!("`{title:?}`s should begin with lowercase"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitleConfig {
    allowed_prefixes: Vec<CompactStr>,
    ignore: Vec<CompactStr>,
    ignore_top_level_describe: bool,
    lowercase_first_character_only: bool,
}

impl std::ops::Deref for PreferLowercaseTitle {
    type Target = PreferLowercaseTitleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitle(Box<PreferLowercaseTitleConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `it`, `test`, `describe`, and `bench` to have descriptions that begin with a
    /// lowercase letter. This provides more readable test failures. This rule is not
    /// enabled by default.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// it('Adds 1 + 2 to equal 3', () => {
    ///     expect(sum(1, 2)).toBe(3);
    /// });
    ///
    /// // valid
    /// it('adds 1 + 2 to equal 3', () => {
    ///     expect(sum(1, 2)).toBe(3);
    /// });
    /// ```
    ///
    /// ## Options
    /// ```json
    /// {
    ///     "jest/prefer-lowercase-title": [
    ///         "error",
    ///         {
    ///             "ignore": ["describe", "test"]
    ///         }
    ///     ]
    /// }
    /// ```
    ///
    /// ### `ignore`
    ///
    /// This array option controls which Jest or Vitest functions are checked by this rule. There
    /// are four possible values:
    /// - `"describe"`
    /// - `"test"`
    /// - `"it"`
    /// - `"bench"`
    ///
    /// By default, none of these options are enabled (the equivalent of
    /// `{ "ignore": [] }`).
    ///
    /// Example of **correct** code for the `{ "ignore": ["describe"] }` option:
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignore": ["describe"] }] */
    /// describe('Uppercase description');
    /// ```
    ///
    /// Example of **correct** code for the `{ "ignore": ["test"] }` option:
    ///
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignore": ["test"] }] */
    /// test('Uppercase description');
    /// ```
    ///
    /// Example of **correct** code for the `{ "ignore": ["it"] }` option:
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignore": ["it"] }] */
    /// it('Uppercase description');
    /// ```
    ///
    /// ### `allowedPrefixes`
    /// This array option allows specifying prefixes, which contain capitals that titles
    /// can start with. This can be useful when writing tests for API endpoints, where
    /// you'd like to prefix with the HTTP method.
    /// By default, nothing is allowed (the equivalent of `{ "allowedPrefixes": [] }`).
    ///
    /// Example of **correct** code for the `{ "allowedPrefixes": ["GET"] }` option:
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "allowedPrefixes": ["GET"] }] */
    /// describe('GET /live');
    /// ```
    ///
    /// ### `ignoreTopLevelDescribe`
    /// This option can be set to allow only the top-level `describe` blocks to have a
    /// title starting with an upper-case letter.
    /// Example of **correct** code for the `{ "ignoreTopLevelDescribe": true }` option:
    ///
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignoreTopLevelDescribe": true }] */
    /// describe('MyClass', () => {
    ///     describe('#myMethod', () => {
    ///         it('does things', () => {
    ///             //
    ///         });
    ///     });
    /// });
    /// ```
    ///
    /// ### `lowercaseFirstCharacterOnly`
    /// This option can be set to only validate that the first character of a test name is lowercased.
    ///
    /// Example of **correct** code for the `{ "lowercaseFirstCharacterOnly": true }` option:
    ///
    /// ```js
    /// /* eslint vitest/prefer-lowercase-title: ["error", { "lowercaseFirstCharacterOnly": true }] */
    /// describe('myClass', () => {
    ///     describe('myMethod', () => {
    ///         it('does things', () => {
    ///             //
    ///         });
    ///     });
    /// });
    /// ```
    ///
    /// Example of **incorrect** code for the `{ "lowercaseFirstCharacterOnly": true }` option:
    ///
    /// ```js
    /// /* eslint vitest/prefer-lowercase-title: ["error", { "lowercaseFirstCharacterOnly": true }] */
    /// describe('MyClass', () => {
    ///     describe('MyMethod', () => {
    ///         it('does things', () => {
    ///             //
    ///         });
    ///     });
    /// });
    /// ```
    PreferLowercaseTitle,
    jest,
    style,
    fix
);

impl Rule for PreferLowercaseTitle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        let ignore_top_level_describe = obj
            .and_then(|config| config.get("ignoreTopLevelDescribe"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let lowercase_first_character_only = obj
            .and_then(|config| config.get("lowercaseFirstCharacterOnly"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let ignore = obj
            .and_then(|config| config.get("ignore"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();
        let allowed_prefixes = obj
            .and_then(|config| config.get("allowedPrefixes"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(PreferLowercaseTitleConfig {
            allowed_prefixes,
            ignore,
            ignore_top_level_describe,
            lowercase_first_character_only,
        }))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ParsedJestFnCallNew::GeneralJest(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let scopes = ctx.scopes();

        let ignores = Self::populate_ignores(&self.ignore);

        if ignores.contains(&jest_fn_call.name.as_ref()) {
            return;
        }

        if matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)) {
            if self.ignore_top_level_describe && scopes.get_flags(node.scope_id()).is_top() {
                return;
            }
        } else if !matches!(
            jest_fn_call.kind,
            JestFnKind::General(JestGeneralFnKind::Test | JestGeneralFnKind::Bench)
        ) {
            return;
        }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        if let Argument::StringLiteral(string_expr) = arg {
            self.lint_string(ctx, string_expr.value.as_str(), string_expr.span);
        } else if let Argument::TemplateLiteral(template_expr) = arg {
            let Some(template_string) = template_expr.quasi() else {
                return;
            };
            self.lint_string(ctx, template_string.as_str(), template_expr.span);
        }
    }
}

impl PreferLowercaseTitle {
    fn lint_string<'a>(&self, ctx: &LintContext<'a>, literal: &'a str, span: Span) {
        if literal.is_empty()
            || self.allowed_prefixes.iter().any(|name| literal.starts_with(name.as_str()))
        {
            return;
        }

        if self.lowercase_first_character_only {
            let Some(first_char) = literal.chars().next() else {
                return;
            };

            let lower = first_char.to_ascii_lowercase();
            if first_char == lower {
                return;
            }
        } else {
            for n in 0..literal.chars().count() {
                let Some(next_char) = literal.chars().nth(n) else {
                    return;
                };

                let next_lower = next_char.to_ascii_lowercase();

                if next_char != next_lower {
                    break;
                }
            }
        }

        let replacement = if self.lowercase_first_character_only {
            cow_utils::CowUtils::cow_to_ascii_lowercase(&literal.chars().as_str()[0..1])
        } else {
            cow_utils::CowUtils::cow_to_ascii_lowercase(literal)
        };

        #[allow(clippy::cast_possible_truncation)]
        let replacement_len = replacement.len() as u32;

        ctx.diagnostic_with_fix(prefer_lowercase_title_diagnostic(literal, span), |fixer| {
            fixer.replace(Span::sized(span.start + 1, replacement_len), replacement)
        });
    }

    fn populate_ignores(ignore: &[CompactStr]) -> Vec<&str> {
        let mut ignores: Vec<&str> = vec![];
        let test_case_name = ["fit", "it", "xit", "test", "xtest"];
        let describe_alias = ["describe", "fdescribe", "xdescribe"];
        let test_name = "test";
        let it_name = "it";
        let bench_name = "bench";

        if ignore.iter().any(|alias| alias == "describe") {
            ignores.extend(describe_alias.iter());
        }

        if ignore.iter().any(|alias| alias == bench_name) {
            ignores.push(bench_name);
        }

        if ignore.iter().any(|alias| alias == test_name) {
            ignores.extend(test_case_name.iter().filter(|alias| alias.ends_with(test_name)));
        }

        if ignore.iter().any(|alias| alias == it_name) {
            ignores.extend(test_case_name.iter().filter(|alias| alias.ends_with(it_name)));
        }

        ignores
    }
}
