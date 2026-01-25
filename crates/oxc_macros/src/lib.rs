//! Macros for declaring lints and secret scanners.
#![warn(missing_docs)]
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod declare_oxc_lint;
mod declare_oxc_shared_lint;

/// Macro used to declare an oxc lint rule
///
/// Every lint declaration consists of 4 parts:
///
/// 1. The documentation
/// 2. The lint's struct
/// 3. The lint's category
/// 4. What kind of auto-fixes the lint supports, if any
///
/// And optionally, a 5th part for defining configuration if there are any config options.
///
/// ## Documentation
/// Lint rule documentation added here will be used to build documentation pages
/// for [our website](https://oxc.rs). Please make sure they are clear and
/// concise. Remember, end users will depend on it to understand the purpose of
/// the lint and how to use it!
///
/// ## Category
/// Please see the [rule category
/// documentation](https://oxc.rs/docs/contribute/linter.html#rule-category) for
/// a full list of categories and their descriptions.
///
/// ## Auto-fixes
///
/// Lints that support auto-fixes **must** specify what kind of auto-fixes they
/// support. Here are some examples:
/// - `none`: no auto-fixes are available (default)
/// - `pending`: auto-fixes are planned but not yet implemented
/// - `fix`: safe, automatic fixes are available
/// - `suggestion`: automatic fixes are available, but may not be safe
/// - `conditional_fix`: automatic fixes are available in some cases
/// - `dangerous_fix`: automatic fixes are available, but may be dangerous
///
/// More generally, auto-fix categories follow this pattern:
/// ```text
/// [conditional?]_[fix|suggestion|dangerous]
/// ```
/// ...meaning that these are also valid:
/// - `suggestion_fix` (supports safe fixes and suggestions)
/// - `conditional_dangerous_fix` (sometimes provides dangerous fixes)
/// - `dangerous_fix_dangerous_suggestion` (provides dangerous fixes and suggestions in all cases)
///
/// `pending` and `none` are special cases that do not follow this pattern.
///
/// ## Integration markers
/// You can optionally add an integration marker immediately after the rule's struct
/// name in parentheses. Currently the only supported marker is `tsgolint`:
///
/// ```rust,ignore
/// declare_oxc_lint!(
///     /// Docs...
///     MyRule(tsgolint),
///     eslint,
///     style,
///     fix
/// );
/// ```
///
/// Adding `(tsgolint)` sets an internal `IS_TSGOLINT_RULE` flag to `true`, which
/// allows the `oxlint` CLI to surface this rule to the external `tsgolint`
/// executable. Rules without the marker keep the default `false` value and are
/// ignored by that integration. Only one marker is allowed and any other value
/// will result in a compile error.
///
/// # Example
///
/// ```
/// use oxc_macros::declare_oxc_lint;
///
/// #[derive(Debug, Default, Clone)]
/// pub struct NoDebugger(Box<NoDebuggerConfig>);
///
/// #[derive(Debug, Default, Clone, JsonSchema)]
/// #[serde(rename_all = "camelCase", default)]
/// pub struct NoDebuggerConfig {
///    /// Explanation for the config goes here.
///    allow: Vec<CompactStr>,
/// }
///
/// declare_oxc_lint!(
///     /// ### What it does
///     ///
///     /// Checks for usage of the `debugger` statement
///     ///
///     /// ### Why is this bad?
///     ///
///     /// `debugger` statements do not affect functionality when a debugger isn't attached.
///     /// They're most commonly an accidental debugging leftover.
///     ///
///     /// ### Examples
///     ///
///     /// Examples of **incorrect** code for this rule:
///     /// ```js
///     /// const data = await getData();
///     /// const result = complexCalculation(data);
///     /// debugger;
///     /// ```
///     ///
///     /// Examples of **correct** code for this rule:
///     /// ```js
///     /// // Not a debugger statement
///     /// var debug = require('foo');
///     /// ```
///     NoDebugger,
///     eslint,
///     correctness,
///     fix,
///     config = NoDebuggerConfig,
/// );
/// ```
#[proc_macro]
pub fn declare_oxc_lint(input: TokenStream) -> TokenStream {
    let metadata = parse_macro_input!(input as declare_oxc_lint::LintRuleMeta);
    declare_oxc_lint::declare_oxc_lint(metadata)
}

/// Same as `declare_oxc_lint`, but doesn't do imports.
/// Enables multiple usages in a single file.
#[proc_macro]
pub fn declare_oxc_lint_test(input: TokenStream) -> TokenStream {
    let mut metadata = parse_macro_input!(input as declare_oxc_lint::LintRuleMeta);
    metadata.used_in_test = true;
    declare_oxc_lint::declare_oxc_lint(metadata)
}

/// Macro used to declare a shared lint rule that references documentation from a shared location
///
/// This macro is similar to `declare_oxc_lint!` but allows you to reference
/// documentation from a shared module, avoiding duplication when the same rule
/// exists in multiple plugins.
///
/// Every shared lint declaration consists of 4 required parts and optional configuration:
///
/// 1. The lint's struct name
/// 2. The lint's plugin
/// 3. The lint's category
/// 4. What kind of auto-fixes the lint supports, if any
/// 5. A `shared_docs` parameter pointing to the shared documentation module
///
/// And optionally, a `config` parameter for defining configuration options.
///
/// ## Shared Documentation
/// The `shared_docs` parameter should point to a module path that exports a
/// `DOCUMENTATION` constant containing the rule's documentation string.
/// This allows multiple plugin implementations to reference the same documentation.
///
/// ## Plugin and Category
/// These work the same as in `declare_oxc_lint!`. Each usage of the macro
/// specifies one plugin (e.g., `jest` or `vitest`).
///
/// ## Auto-fixes
/// These work the same as in `declare_oxc_lint!`. See that macro's documentation for details.
///
/// # Example
///
/// In `shared/valid_title.rs`:
/// ```rust,ignore
/// #[cfg(feature = "ruledocs")]
/// pub const DOCUMENTATION: Option<&str> = Some(
///     r#"
/// ### What it does
///
/// Checks that the titles of Jest and Vitest blocks are valid.
///
/// ### Why is this bad?
///
/// Titles that are not valid can be misleading.
/// "#
/// );
/// ```
///
/// In `jest/valid_title.rs`:
/// ```rust,ignore
/// use oxc_macros::declare_oxc_shared_lint;
///
/// #[derive(Debug, Default, Clone)]
/// pub struct ValidTitle(Box<SharedValidTitle::ValidTitleConfig>);
///
/// declare_oxc_shared_lint!(
///     ValidTitle,
///     jest,
///     correctness,
///     conditional_fix,
///     shared_docs = crate::rules::shared::valid_title
/// );
/// ```
///
/// In `vitest/valid_title.rs`:
/// ```rust,ignore
/// use oxc_macros::declare_oxc_shared_lint;
///
/// #[derive(Debug, Default, Clone)]
/// pub struct ValidTitle(Box<SharedValidTitle::ValidTitleConfig>);
///
/// declare_oxc_shared_lint!(
///     ValidTitle,
///     vitest,
///     correctness,
///     conditional_fix,
///     shared_docs = crate::rules::shared::valid_title
/// );
/// ```
#[proc_macro]
pub fn declare_oxc_shared_lint(input: TokenStream) -> TokenStream {
    let metadata = parse_macro_input!(input as declare_oxc_shared_lint::SharedLintRuleMeta);
    declare_oxc_shared_lint::declare_oxc_shared_lint(metadata)
}
