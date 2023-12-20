use oxc_ast::{
    ast::{Argument, Expression, RegExpFlags},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::{extract_regex_flags, get_declaration_of_variable, is_method_call},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("deepscan(bad-replace-all-arg): Global flag (g) is missing in the regular expression supplied to the `replaceAll` method.")]
#[diagnostic(severity(warning), help("To replace all occurrences of a string, use the `replaceAll` method with the global flag (g) in the regular expression."))]
struct BadReplaceAllArgDiagnostic(
    #[label("`replaceAll` called here")] pub Span,
    #[label("RegExp supplied here")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct BadReplaceAllArg;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when the `replaceAll` method is called with a regular expression that does not have the global flag (g).
    ///
    /// ### Why is this bad?
    ///
    /// The `replaceAll` method replaces all occurrences of a string with another string. If the global flag (g) is not used in the regular expression, only the first occurrence of the string will be replaced.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad: The global flag (g) is missing in the regular expression.
    /// withSpaces.replaceAll(/\s+/, ',');
    ///
    /// // Good: The global flag (g) is used in the regular expression.
    /// withSpaces.replaceAll(/\s+/g, ',');
    /// ```
    BadReplaceAllArg,
    correctness
);

impl Rule for BadReplaceAllArg {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        if !is_method_call(call_expr, None, Some(&["replaceAll"]), Some(1), None) {
            return;
        }

        let Argument::Expression(regexp_argument) = &call_expr.arguments[0] else {
            return;
        };

        let Some((flags, regex_span)) = resolve_flags(regexp_argument, ctx) else {
            return;
        };

        if !flags.contains(RegExpFlags::G) {
            let Expression::MemberExpression(call_expr_callee) = &call_expr.callee else { return };
            let Some((replace_all_span, _)) = call_expr_callee.static_property_info() else {
                return;
            };

            ctx.diagnostic(BadReplaceAllArgDiagnostic(replace_all_span, regex_span));
        }
    }
}

fn resolve_flags<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<(RegExpFlags, Span)> {
    match expr.without_parenthesized() {
        Expression::RegExpLiteral(regexp_literal) => {
            Some((regexp_literal.regex.flags, regexp_literal.span))
        }
        Expression::NewExpression(new_expr) => {
            if new_expr.callee.is_specific_id("RegExp") {
                Some((
                    extract_regex_flags(&new_expr.arguments).unwrap_or(RegExpFlags::empty()),
                    new_expr.span,
                ))
            } else {
                None
            }
        }
        Expression::Identifier(ident) => {
            if let Some(decl) = get_declaration_of_variable(ident, ctx) {
                if let AstKind::VariableDeclarator(var_decl) = decl.kind() {
                    if let Some(init) = &var_decl.init {
                        return resolve_flags(init, ctx);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // valid call
        r"withSpaces.replaceAll(/\s+/g, ',');",
        // incorrect number of arguments
        r"withSpaces.replaceAll();",
        // not a method call
        r"replaceAll(/\s+/, ',');",
        // not a method call
        r"withSpaces();",
        // new RegExp
        r"withSpaces.replaceAll(new RegExp('\s+', 'g'), ',');",
        // new replaceAll
        r"new replaceAll(/\s+/, ',');",
        // resolved vars
        r#"const foo = "string"; withSpaces.replaceAll(foo, ',');"#,
        // resolved vars
        r"const foo = /\s+/g; withSpaces.replaceAll(foo, ',');",
        // resolved vars
        r"const foo = new RegExp('\s+', 'g'); withSpaces.replaceAll(foo, ',');",
    ];

    let fail = vec![
        r"withSpaces.replaceAll(/\s+/, ',');",
        r"withSpaces.replaceAll(/\s+/i, ',');",
        r"withSpaces.replaceAll(new RegExp('\s+'), ',');",
        r"withSpaces.replaceAll(new RegExp('\s+','i'), ',');",
        // resolved vars
        r"
            const foo = /\s+/;
            
            withSpaces.replaceAll(foo, ',');
        ",
        r"
            const foo = /\s+/i;
            
            withSpaces.replaceAll(foo, ',');
        ",
        r"
            const foo = new RegExp('\s+');
        
            withSpaces.replaceAll(foo, ',');
        ",
    ];

    Tester::new_without_config(BadReplaceAllArg::NAME, pass, fail).test_and_snapshot();
}
