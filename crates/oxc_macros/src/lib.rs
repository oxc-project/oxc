use syn::parse_macro_input;
use typename::impl_typename_macro;

mod declare_all_lint_rules;
mod declare_oxc_lint;
mod typename;

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
pub fn declare_oxc_lint(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let metadata = parse_macro_input!(input as declare_oxc_lint::LintRuleMeta);

    declare_oxc_lint::declare_oxc_lint(metadata).into()
}

/// Same as `declare_oxc_lint`, but doesn't do imports.
/// Enables multiple usages in a single file.
#[proc_macro]
pub fn declare_oxc_lint_test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut metadata = parse_macro_input!(input as declare_oxc_lint::LintRuleMeta);
    metadata.used_in_test = true;

    declare_oxc_lint::declare_oxc_lint(metadata).into()
}

#[proc_macro]
pub fn declare_all_lint_rules(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let metadata = parse_macro_input!(input as declare_all_lint_rules::AllLintRulesMeta);

    declare_all_lint_rules::declare_all_lint_rules(metadata).into()
}

/// Adds an implementation of Trustfall's Typename trait to a struct based on the
/// struct's name. If the struct was named `ClassVertex`, the implementation would
/// implement the function to return "ClassAST" if `self.ast_node.is_some()`
/// and "Class" otherwise.
#[proc_macro_derive(TypeName)]
pub fn typename_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).expect("to be able to parse the input into a struct");

    // Build the trait implementation
    impl_typename_macro(&ast)
}
