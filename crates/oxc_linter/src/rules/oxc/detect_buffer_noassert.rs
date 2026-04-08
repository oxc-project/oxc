use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_buffer_noassert_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Buffer read/write called with noAssert flag")
        .with_help("Do not pass true as the last argument (noAssert) to Buffer read/write methods. This disables bounds checking and can lead to out-of-bounds access.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectBufferNoassert;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects calls to Buffer read/write methods with the `noAssert` flag set to `true`.
    ///
    /// ### Why is this bad?
    ///
    /// Passing `true` as the last argument to Buffer read/write methods disables
    /// bounds checking. This can lead to out-of-bounds reads and writes, potentially
    /// exposing sensitive memory or causing crashes.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// buf.readUInt8(0, true);
    /// buf.writeUInt32BE(value, 0, true);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// buf.readUInt8(0);
    /// buf.writeUInt32BE(value, 0);
    /// ```
    DetectBufferNoassert,
    oxc,
    suspicious,
    none
);

const BUFFER_METHODS: &[&str] = &[
    "readUInt8",
    "readUInt16LE",
    "readUInt16BE",
    "readUInt32LE",
    "readUInt32BE",
    "readInt8",
    "readInt16LE",
    "readInt16BE",
    "readInt32LE",
    "readInt32BE",
    "readFloatLE",
    "readFloatBE",
    "readDoubleLE",
    "readDoubleBE",
    "writeUInt8",
    "writeUInt16LE",
    "writeUInt16BE",
    "writeUInt32LE",
    "writeUInt32BE",
    "writeInt8",
    "writeInt16LE",
    "writeInt16BE",
    "writeInt32LE",
    "writeInt32BE",
    "writeFloatLE",
    "writeFloatBE",
    "writeDoubleLE",
    "writeDoubleBE",
];

impl Rule for DetectBufferNoassert {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call_expr.callee else {
            return;
        };

        if !BUFFER_METHODS.contains(&member.property.name.as_str()) {
            return;
        }

        let Some(last_arg) = call_expr.arguments.last().and_then(|a| a.as_expression()) else {
            return;
        };

        if let Expression::BooleanLiteral(lit) = last_arg
            && lit.value
        {
            ctx.diagnostic(detect_buffer_noassert_diagnostic(call_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "buf.readUInt8(0)",
        "buf.writeUInt32BE(value, 0)",
        "buf.readUInt8(0, false)",
        "buf.foo(0, true)",
        "obj.someMethod(true)",
    ];

    let fail = vec![
        "buf.readUInt8(0, true)",
        "buf.readUInt16LE(0, true)",
        "buf.readUInt16BE(0, true)",
        "buf.readUInt32LE(0, true)",
        "buf.readUInt32BE(0, true)",
        "buf.readInt8(0, true)",
        "buf.readInt16LE(0, true)",
        "buf.readInt16BE(0, true)",
        "buf.readInt32LE(0, true)",
        "buf.readInt32BE(0, true)",
        "buf.readFloatLE(0, true)",
        "buf.readFloatBE(0, true)",
        "buf.readDoubleLE(0, true)",
        "buf.readDoubleBE(0, true)",
        "buf.writeUInt8(0, 0, true)",
        "buf.writeUInt16LE(0, 0, true)",
        "buf.writeUInt16BE(0, 0, true)",
        "buf.writeUInt32LE(0, 0, true)",
        "buf.writeUInt32BE(0, 0, true)",
        "buf.writeInt8(0, 0, true)",
        "buf.writeInt16LE(0, 0, true)",
        "buf.writeInt16BE(0, 0, true)",
        "buf.writeInt32LE(0, 0, true)",
        "buf.writeInt32BE(0, 0, true)",
        "buf.writeFloatLE(0, 0, true)",
        "buf.writeFloatBE(0, 0, true)",
        "buf.writeDoubleLE(0, 0, true)",
        "buf.writeDoubleBE(0, 0, true)",
    ];

    Tester::new(DetectBufferNoassert::NAME, DetectBufferNoassert::PLUGIN, pass, fail)
        .test_and_snapshot();
}
