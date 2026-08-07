//! Statement-leading comments whose statement dissolves into an expression
//! operand must not print in that operand. The parser marks their origin and
//! codegen's operand printers keep them out without minifier cooperation.

use crate::test;

/// test262 `language/asi/S7.9_A5.8_T1.js`: the banner leads the second
/// assignment statement, which fuses into the merged `if` test.
#[test]
fn drops_banner_of_fused_statement() {
    test(
        "var z = 0, x = 0, y = 0;
z = x + ++y;
if (z !== 2 && x !== 0) throw new Test262Error('');
// banner
z = x + ++y;
if (z !== 3 && x !== 0) throw new Test262Error('');",
        "var z = 0, x = 0, y = 0;
if (z = x + ++y, z !== 2 && x !== 0 || (z = x + ++y, z !== 3 && x !== 0)) throw new Test262Error('');",
    );
}

/// webpack `CssParser.js`: a JSDoc cast leads the `if` body, which becomes
/// the `&&` right-hand side.
#[test]
fn drops_jsdoc_of_if_body_converted_to_logical() {
    test(
        "if (module.exportType === 'style')
	/** @type {BuildMeta} */
	(module.buildMeta).needIdInConcatenation = true;",
        "module.exportType === 'style' && (module.buildMeta.needIdInConcatenation = !0);",
    );
}

#[test]
fn drops_comments_of_if_else_converted_to_conditional() {
    test(
        "if (a) /* yes */ b();
else /* no */ c();",
        "a ? b() : c();",
    );
}

/// Annotations retain expression-level meaning when their statement dissolves.
#[test]
fn keeps_annotation_of_dissolved_statement() {
    test("if (a) /* istanbul ignore next */ b();", "a && /* istanbul ignore next */ b();");
}
