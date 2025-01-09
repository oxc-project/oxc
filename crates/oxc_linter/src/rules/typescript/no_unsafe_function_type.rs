use oxc_ast::{
    ast::{Expression, IdentifierReference, TSTypeName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_unsafe_function_type_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("The `Function` type accepts any function-like value.")
        .with_help("Prefer explicitly defining any function parameters and return type.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeFunctionType;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow using the unsafe built-in Function type.
    ///
    /// ### Why is this bad?
    /// TypeScript's built-in Function type allows being called with any number of arguments and returns type any. Function also allows classes or plain objects that happen to possess all properties of the Function class. It's generally better to specify function parameters and return types with the function type syntax.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let noParametersOrReturn: Function;
    ///  noParametersOrReturn = () => {};
    ///
    ///  let stringToNumber: Function;
    ///  stringToNumber = (text: string) => text.length;
    ///
    ///  let identity: Function;
    ///  identity = value => value;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let noParametersOrReturn: () => void;
    /// noParametersOrReturn = () => {};
    ///
    /// let stringToNumber: (text: string) => number;
    /// stringToNumber = text => text.length;
    ///
    /// let identity: <T>(value: T) => T;
    /// identity = value => value;
    /// ```
    NoUnsafeFunctionType,
    typescript,
    pedantic,
);

impl Rule for NoUnsafeFunctionType {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSTypeReference(reference) => {
                if let TSTypeName::IdentifierReference(ident_ref) = &reference.type_name {
                    handle_function_type(ident_ref, ctx);
                }
            }
            AstKind::TSClassImplements(implements) => {
                if let TSTypeName::IdentifierReference(ident_ref) = &implements.expression {
                    handle_function_type(ident_ref, ctx);
                }
            }
            AstKind::TSInterfaceHeritage(heritage) => {
                if let Expression::Identifier(ident) = &heritage.expression {
                    handle_function_type(ident, ctx);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn handle_function_type<'a>(identifier: &'a IdentifierReference<'a>, ctx: &LintContext<'a>) {
    if identifier.name == "Function" && identifier.is_global_reference(ctx.symbols()) {
        ctx.diagnostic(no_unsafe_function_type_diagnostic(identifier.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let value: () => void;",
        "let value: <T>(t: T) => T;",
        "
            // create a scope since it's illegal to declare a duplicate identifier
            // 'Function' in the global script scope.
            {
              type Function = () => void;
              let value: Function;
            }
          ",
    ];

    let fail = vec![
        "let value: Function;",
        "let value: Function[];",
        "let value: Function | number;",
        "
			        class Weird implements Function {
			          // ...
			        }
			      ",
        "
			        interface Weird extends Function {
			          // ...
			        }
			      ",
    ];

    Tester::new(NoUnsafeFunctionType::NAME, NoUnsafeFunctionType::PLUGIN, pass, fail)
        .test_and_snapshot();
}
