use oxc_ast::{
    AstKind,
    ast::{Expression, TemplateLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{Ident, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_path_concat_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `path.join()` or `path.resolve()` instead of string concatenation")
        .with_help("Replace string concatenation of `__dirname` or `__filename` with `path.join()` or `path.resolve()`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoPathConcat;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows string concatenation with `__dirname` and `__filename`.
    ///
    /// ### Why is this bad?
    ///
    /// In Node.js, the `__dirname` and `__filename` global variables contain the directory path and the file path of the currently executing script file, respectively.
    /// Sometimes, developers try to use these variables to create paths to other files, such as:
    ///
    /// ```js
    /// var fullPath = __dirname + "/foo.js";
    /// ```
    ///
    /// However, this is error-prone because it doesn't account for different
    /// operating systems, which use different path separators. Using `path.join()`
    /// or `path.resolve()` is the proper way to create cross-platform file paths.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const fullPath1 = __dirname + "/foo.js";
    /// const fullPath2 = __filename + "/foo.js";
    /// const fullPath3 = `${__dirname}/foo.js`;
    /// const fullPath4 = `${__filename}/foo.js`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const fullPath1 = path.join(__dirname, "foo.js");
    /// const fullPath2 = path.join(__filename, "foo.js");
    /// const fullPath3 = __dirname + ".js";
    /// const fullPath4 = __filename + ".map";
    /// const fullPath5 = `${__dirname}_foo.js`;
    /// const fullPath6 = `${__filename}.test.js`;
    /// ```
    NoPathConcat,
    node,
    restriction
);

impl Rule for NoPathConcat {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                if bin_expr.operator != BinaryOperator::Addition {
                    return;
                }

                if is_dirname_or_filename(&bin_expr.left, ctx)
                    && starts_with_path_separator(&bin_expr.right)
                {
                    ctx.diagnostic(no_path_concat_diagnostic(bin_expr.span));
                }
            }
            AstKind::TemplateLiteral(temp_lit) => {
                for (i, expr) in temp_lit.expressions.iter().enumerate() {
                    if is_dirname_or_filename(expr, ctx)
                        && template_element_starts_with_path_separator(temp_lit, i + 1)
                    {
                        ctx.diagnostic(no_path_concat_diagnostic(temp_lit.span));
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_path_sep(expr: &Expression) -> bool {
    expr.is_specific_member_access("path", "sep")
}

fn is_path_separator(c: char) -> bool {
    c == '/' || c == '\\'
}

fn is_dirname_or_filename(expr: &Expression, ctx: &LintContext) -> bool {
    let Expression::Identifier(ident) = expr else {
        return false;
    };
    ident.is_global_reference_name(Ident::new_const("__dirname"), ctx.scoping())
        || ident.is_global_reference_name(Ident::new_const("__filename"), ctx.scoping())
}

fn starts_with_path_separator(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral(s) => s.value.chars().next().is_some_and(is_path_separator),
        Expression::TemplateLiteral(temp_lit) => {
            template_element_starts_with_path_separator(temp_lit, 0)
        }
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            starts_with_path_separator(&bin.left)
        }
        Expression::ConditionalExpression(cond) => {
            starts_with_path_separator(&cond.consequent)
                || starts_with_path_separator(&cond.alternate)
        }
        Expression::LogicalExpression(logical) => {
            starts_with_path_separator(&logical.left) || starts_with_path_separator(&logical.right)
        }
        Expression::AssignmentExpression(assign) => starts_with_path_separator(&assign.right),
        Expression::SequenceExpression(seq) => {
            seq.expressions.last().is_some_and(|last| starts_with_path_separator(last))
        }
        Expression::ParenthesizedExpression(paren) => starts_with_path_separator(&paren.expression),
        _ => is_path_sep(expr),
    }
}

fn template_element_starts_with_path_separator(temp_lit: &TemplateLiteral, i: usize) -> bool {
    let Some(quasi) = temp_lit.quasis.get(i) else {
        return false;
    };

    if let Some(c) = quasi.value.cooked.as_ref().and_then(|cooked| cooked.chars().next())
        && is_path_separator(c)
    {
        return true;
    }

    temp_lit.expressions.get(i).is_some_and(|expr| starts_with_path_separator(expr))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"var fullPath = dirname + "foo.js";"#,
        r#"var fullPath = __dirname == "foo.js";"#,
        "if (fullPath === __dirname) {}",
        "if (__dirname === fullPath) {}",
        r#"var fullPath = "/foo.js" + __filename;"#,
        r#"var fullPath = "/foo.js" + __dirname;"#,
        r#"var fullPath = __filename + ".map";"#,
        "var fullPath = `${__filename}.map`;",
        r#"var fullPath = __filename + (test ? ".js" : ".ts");"#,
        r#"var fullPath = __filename + (ext || ".js");"#,
        r"var fullPath = `${__dirname}\nfoo.js`;",
    ];

    let fail = vec![
        r#"var fullPath = __dirname + "/foo.js";"#,
        r#"var fullPath = __filename + "/foo.js";"#,
        "var fullPath = `${__dirname}/foo.js`;",
        "var fullPath = `${__filename}/foo.js`;",
        r#"var path = require("path"); var fullPath = `${__dirname}${path.sep}foo.js`;"#,
        r#"var path = require("path"); var fullPath = `${__filename}${path.sep}foo.js`;"#,
        r#"var path = require("path"); var fullPath = __dirname + path.sep + `foo.js`;"#,
        r#"var fullPath = __dirname + "/" + "foo.js";"#,
        r#"var fullPath = __dirname + ("/" + "foo.js");"#,
        r#"var fullPath = __dirname + (test ? "/foo.js" : "/bar.js");"#,
        r#"var fullPath = __dirname + (extraPath || "/default.js");"#,
        r#"var fullPath = __dirname + "\\foo.js";"#,
        r#"var fullPath = __dirname + "\\${path.sep}foo.js";"#,
        r#"var fullPath = __filename + "\\${path.sep}foo.js";"#,
        r"var fullPath = `${__dirname}\\${path.sep}foo.js`;",
        r"var fullPath = `${__filename}\\${path.sep}foo.js`;",
    ];

    Tester::new(NoPathConcat::NAME, NoPathConcat::PLUGIN, pass, fail).test_and_snapshot();
}
