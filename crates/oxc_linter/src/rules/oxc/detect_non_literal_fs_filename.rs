use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_non_literal_fs_filename_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("fs.{method}() called with a non-literal file path"))
        .with_help("Avoid passing dynamic values as file paths to fs methods. This can allow an attacker to access arbitrary files on the file system.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectNonLiteralFsFilename;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects calls to `fs` methods where the filename argument is not a string literal.
    ///
    /// ### Why is this bad?
    ///
    /// Passing user-controlled values as file paths to `fs` methods can allow
    /// path traversal attacks, enabling an attacker to read, write, or delete
    /// arbitrary files on the file system.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// fs.readFile(userInput, callback);
    /// fs.writeFileSync(filePath, data);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// fs.readFile("./config.json", callback);
    /// fs.writeFileSync("output.txt", data);
    /// ```
    DetectNonLiteralFsFilename,
    oxc,
    suspicious,
    none
);

const FS_METHODS: &[&str] = &[
    "access",
    "appendFile",
    "chmod",
    "chown",
    "close",
    "copyFile",
    "createReadStream",
    "createWriteStream",
    "exists",
    "fchmod",
    "fchown",
    "fdatasync",
    "fstat",
    "fsync",
    "ftruncate",
    "futimes",
    "lchmod",
    "lchown",
    "link",
    "lstat",
    "mkdir",
    "mkdtemp",
    "open",
    "read",
    "readFile",
    "readdir",
    "readlink",
    "realpath",
    "rename",
    "rmdir",
    "stat",
    "symlink",
    "truncate",
    "unlink",
    "unwatchFile",
    "utimes",
    "watch",
    "watchFile",
    "write",
    "writeFile",
    // Sync variants
    "accessSync",
    "appendFileSync",
    "chmodSync",
    "chownSync",
    "closeSync",
    "copyFileSync",
    "existsSync",
    "fchmodSync",
    "fchownSync",
    "fdatasyncSync",
    "fstatSync",
    "fsyncSync",
    "ftruncateSync",
    "futimesSync",
    "lchmodSync",
    "lchownSync",
    "linkSync",
    "lstatSync",
    "mkdirSync",
    "mkdtempSync",
    "openSync",
    "readFileSync",
    "readSync",
    "readdirSync",
    "readlinkSync",
    "realpathSync",
    "renameSync",
    "rmdirSync",
    "statSync",
    "symlinkSync",
    "truncateSync",
    "unlinkSync",
    "utimesSync",
    "writeFileSync",
    "writeSync",
];

impl Rule for DetectNonLiteralFsFilename {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call_expr.callee else {
            return;
        };

        let method_name = member.property.name.as_str();
        if !FS_METHODS.contains(&method_name) {
            return;
        }

        let Some(arg) = call_expr.arguments.first().and_then(|a| a.as_expression()) else {
            return;
        };

        match arg {
            Expression::StringLiteral(_) => return,
            Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => return,
            _ => {}
        }

        ctx.diagnostic(detect_non_literal_fs_filename_diagnostic(call_expr.span, method_name));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"fs.readFile("./config.json", cb)"#,
        r#"fs.writeFileSync("output.txt", data)"#,
        "fs.readFile(`static-path`, cb)",
        "obj.customMethod(variable)",
        "fs.readFile()",
    ];

    let fail = vec![
        "fs.readFile(userInput, cb)",
        "fs.writeFileSync(filePath, data)",
        "fs.readFile(getPath(), cb)",
        "fs.unlink(dynamicFile, cb)",
        "fs.readFile(`${dir}/file`, cb)",
    ];

    Tester::new(DetectNonLiteralFsFilename::NAME, DetectNonLiteralFsFilename::PLUGIN, pass, fail)
        .test_and_snapshot();
}
