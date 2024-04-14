use proc_macro::TokenStream;
use syn::parse_macro_input;

mod declare_all_lint_rules;
mod declare_oxc_lint;

/// Macro used to declare an oxc lint rule
///
/// Every lint declaration consists of 2 parts:
///
/// 1. The documentation
/// 2. The lint's struct
///
/// # Example
///
/// ```
/// use oxc_macros::declare_oxc_lint;
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
///     /// ### Example
///     /// ```javascript
///     /// const data = await getData();
///     /// const result = complexCalculation(data);
///     /// debugger;
///     /// ```
///     ///
///     /// ```
///     pub struct NoDebugger
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

#[proc_macro]
pub fn declare_all_lint_rules(input: TokenStream) -> TokenStream {
    let metadata = parse_macro_input!(input as declare_all_lint_rules::AllLintRulesMeta);
    declare_all_lint_rules::declare_all_lint_rules(metadata)
}
