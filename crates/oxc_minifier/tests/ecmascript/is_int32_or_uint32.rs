use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_ecmascript::{WithoutGlobalReferenceInformation, constant_evaluation::IsInt32OrUint32};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn test(source_text: &str, expected: bool) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    let result = stmt.expression.is_int32_or_uint32(&WithoutGlobalReferenceInformation {});
    assert_eq!(result, expected, "{source_text}");
}

#[test]
fn literal_tests() {
    test("1n", false);
    test("true", false);
    test("null", false);
    test("0", true);
    test("-0", false);
    test("0.1", false);
    test("0/0", false);
    test("1/0", false);
    test("-1/0", false);
    test("('')", false);
}

#[test]
fn unary_tests() {
    test("~foo", false); // maybe bigint
    test("~+foo", true);
    test("~0", true);

    test("+foo", false); // maybe non-int32/uint32 number
    test("+0", true);

    test("void 0", false);
    test("-foo", false);
    test("!foo", false);
    test("delete foo", false);
    test("typeof foo", false);
}

#[test]
fn binary_tests() {
    test("foo << bar", false); // maybe bigint
    test("foo << 0", true);
    test("foo >> bar", false); // maybe bigint
    test("foo >> 0", true);
    test("foo >>> bar", true); // always number
    test("foo & bar", false); // maybe bigint
    test("foo & 0", true);
    test("foo | bar", false); // maybe bigint
    test("foo | 0", true);
    test("foo ^ bar", false); // maybe bigint
    test("foo ^ 0", true);

    test("foo + bar", false);
    test("foo - bar", false);
    test("foo * bar", false);
    test("foo / bar", false);
    test("foo % bar", false);
    test("foo ** bar", false);
    test("foo instanceof Object", false);
    test("'foo' in foo", false);
    test("foo == bar", false);
    test("foo != bar", false);
    test("foo === bar", false);
    test("foo !== bar", false);
    test("foo < bar", false);
    test("foo <= bar", false);
    test("foo > bar", false);
    test("foo >= bar", false);
    test("#foo in bar", false);
}

#[test]
fn sequence_tests() {
    test("(1, 0)", true);
    test("(1, foo)", false);
}

#[test]
fn conditional_tests() {
    test("foo ? 0 : 0.1", false);
    test("foo ? 0.1 : 0", false);
    test("foo ? 0 : 1", true);
}

#[test]
fn logical_tests() {
    test("~+foo && 0.1", false);
    test("0.1 && ~+foo", false);
    test("~+foo && 1", true);

    test("~+foo || 0.1", false);
    test("0.1 || ~+foo", false);
    test("~+foo || 0", true);

    test("~+foo ?? 0.1", true);
    test("0.1 ?? ~+foo", false);
}

#[test]
fn undetermined_tests() {
    test("foo", false);
}
