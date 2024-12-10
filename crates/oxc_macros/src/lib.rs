//! Macros for declaring lints and secret scanners.
#![warn(missing_docs)]
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod declare_all_lint_rules;
mod declare_oxc_lint;

/// Macro used to declare an oxc lint rule
///
/// Every lint declaration consists of 4 parts:
///
/// 1. The documentation
/// 2. The lint's struct
/// 3. The lint's category
/// 4. What kind of auto-fixes the lint supports
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
/// # Example
///
/// ```
/// use oxc_macros::declare_oxc_lint;
///
/// #[derive(Debug, Default, Clone)]
/// pub struct NoDebugger;
///
/// declare_oxc_lint! {
///     /// ### What it does
///     /// Checks for usage of the `debugger` statement
///     ///
///     /// ### Why is this bad?
///     /// `debugger` statements do not affect functionality when a debugger isn't attached.
///     /// They're most commonly an accidental debugging leftover.
///     ///
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
///     correctness,
///     fix
/// }
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

/// Declare all lint rules in a single macro. This create the `RuleEnum` struct,
/// which is effectively a compile-time v-table for all lint rules. This
/// bypasses object-safety requirements and allows for compile-time dispatch
/// over a heterogeneous set of known lint rules.
#[proc_macro]
pub fn declare_all_lint_rules(input: TokenStream) -> TokenStream {
    let metadata = parse_macro_input!(input as declare_all_lint_rules::AllLintRulesMeta);
    declare_all_lint_rules::declare_all_lint_rules(metadata)
}
