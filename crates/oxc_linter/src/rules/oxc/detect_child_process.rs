use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_child_process_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("child_process.exec() called with a non-literal argument")
        .with_help("Avoid calling child_process.exec() with dynamic expressions. Use child_process.execFile() or spawn() with an argument array instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectChildProcess;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects calls to `child_process.exec()` with non-literal arguments.
    ///
    /// ### Why is this bad?
    ///
    /// `child_process.exec()` runs a command in a shell. If the command string
    /// is constructed from user input, it can lead to command injection attacks.
    /// Use `child_process.execFile()` or `child_process.spawn()` with an argument
    /// array instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// child_process.exec(userInput);
    /// cp.exec(cmd);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// child_process.exec("ls -la");
    /// child_process.execFile("/bin/ls", ["-la"]);
    /// ```
    DetectChildProcess,
    oxc,
    suspicious,
    none
);

impl Rule for DetectChildProcess {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call_expr.callee else {
            return;
        };

        if member.property.name != "exec" {
            return;
        }

        // Only flag when the object name suggests child_process
        if let Expression::Identifier(obj) = &member.object {
            let name = obj.name.as_str();
            if name != "child_process" && name != "childProcess" && name != "cp" {
                return;
            }
        }

        let Some(arg) = call_expr.arguments.first().and_then(|a| a.as_expression()) else {
            return;
        };

        match arg {
            Expression::StringLiteral(_) => return,
            Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => return,
            _ => {}
        }

        ctx.diagnostic(detect_child_process_diagnostic(call_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"child_process.exec("ls -la")"#,
        r#"cp.exec("echo hello")"#,
        "child_process.exec(`static-command`)",
        "child_process.execFile('/bin/ls', ['-la'])",
        "child_process.spawn('ls', ['-la'])",
        "obj.exec(variable)",
        "db.exec(query)",
    ];

    let fail = vec![
        "child_process.exec(userInput)",
        "cp.exec(cmd)",
        "child_process.exec(getCommand())",
        "child_process.exec(`${cmd} -la`)",
    ];

    Tester::new(DetectChildProcess::NAME, DetectChildProcess::PLUGIN, pass, fail)
        .test_and_snapshot();
}
