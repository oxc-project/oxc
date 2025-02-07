use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_ecmascript::constant_evaluation::ValueType;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn test(source_text: &str, expected: ValueType) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    let result = ValueType::from(stmt.expression.without_parentheses());
    assert_eq!(result, expected, "{source_text}");
}

#[test]
fn literal_tests() {
    test("1n", ValueType::BigInt);
    test("true", ValueType::Boolean);
    test("null", ValueType::Null);
    test("0", ValueType::Number);
    test("('')", ValueType::String);
    test("``", ValueType::String);
    test("({})", ValueType::Object);
    test("[]", ValueType::Object);
    test("[0]", ValueType::Object);
    test("/a/", ValueType::Object);
    test("(function () {})", ValueType::Object);
    // test("(() => {})", ValueType::Object);
    // test("(class {})", ValueType::Object);
}

#[test]
fn identifier_tests() {
    test("undefined", ValueType::Undefined);
    test("NaN", ValueType::Number);
    test("Infinity", ValueType::Number);
    test("foo", ValueType::Undetermined);
}

#[test]
fn unary_tests() {
    test("void 0", ValueType::Undefined);
    test("void foo", ValueType::Undefined);

    test("-0", ValueType::Number);
    test("-Infinity", ValueType::Number);
    test("-0n", ValueType::BigInt);
    test("-true", ValueType::Number); // -1
    test("-''", ValueType::Number); // -0
    test("-'0n'", ValueType::Number); // NaN
    test("-null", ValueType::Number); // -0
    test("-undefined", ValueType::Number); // NaN
    test("-{ valueOf() { return 0n } }", ValueType::Undetermined);
    test("-foo", ValueType::Undetermined); // can be number or bigint

    test("+0", ValueType::Number);
    test("+true", ValueType::Number);
    // this may throw an error (when foo is 1n), but the return value is always a number
    test("+foo", ValueType::Number);

    test("!0", ValueType::Boolean);
    test("!foo", ValueType::Boolean);

    test("delete 0", ValueType::Boolean);
    test("delete foo", ValueType::Boolean);

    test("typeof 0", ValueType::String);
    test("typeof foo", ValueType::String);

    // test("~0", ValueType::Number);
    // test("~0n", ValueType::BigInt);
    test("~foo", ValueType::Undetermined); // can be number or bigint
}

#[test]
fn binary_tests() {
    test("'foo' + 'bar'", ValueType::String);
    test("'foo' + bar", ValueType::String);
    test("foo + 'bar'", ValueType::String);
    test("foo + bar", ValueType::Undetermined);
    // `foo` might be a string and the result may be a number or string
    test("foo + 1", ValueType::Undetermined);
    test("1 + foo", ValueType::Undetermined);
    // the result is number, if both are not string and bigint
    test("true + undefined", ValueType::Number);
    test("true + null", ValueType::Number);
    test("true + 0", ValueType::Number);
    // test("undefined + true", ValueType::Number);
    // test("null + true", ValueType::Number);
    // test("0 + true", ValueType::Number);
    test("true + 0n", ValueType::Undetermined); // throws an error
    test("({} + [])", ValueType::Undetermined);
    test("[] + {}", ValueType::Undetermined);

    test("1 - 0", ValueType::Number);
    test("1 * 0", ValueType::Number);
    // test("foo * bar", ValueType::Undetermined); // number or bigint
    test("1 / 0", ValueType::Number);
    test("1 % 0", ValueType::Number);
    test("1 << 0", ValueType::Number);
    test("1 | 0", ValueType::Number);
    test("1 >> 0", ValueType::Number);
    test("1 ^ 0", ValueType::Number);
    test("1 & 0", ValueType::Number);
    test("1 ** 0", ValueType::Number);
    test("1 >>> 0", ValueType::Number);

    // foo * bar can be number, but the result is always a bigint (if no error happened)
    // test("foo * bar * 1n", ValueType::BigInt);
    test("foo * bar * 1", ValueType::Number);
    // unsigned right shift always returns a number
    test("foo >>> (1n + 0n)", ValueType::Number);

    test("foo instanceof Object", ValueType::Boolean);
    test("'foo' in foo", ValueType::Boolean);
    test("foo == bar", ValueType::Boolean);
    test("foo != bar", ValueType::Boolean);
    test("foo === bar", ValueType::Boolean);
    test("foo !== bar", ValueType::Boolean);
    test("foo < bar", ValueType::Boolean);
    test("foo <= bar", ValueType::Boolean);
    test("foo > bar", ValueType::Boolean);
    test("foo >= bar", ValueType::Boolean);
}

#[test]
fn sequence_tests() {
    test("(1, 2n)", ValueType::BigInt);
    test("(1, foo)", ValueType::Undetermined);
}

#[test]
fn assignment_tests() {
    test("a = 1", ValueType::Number);
    test("a = 1n", ValueType::BigInt);
    test("a = foo", ValueType::Undetermined);

    // test("a += 1", ValueType::Undetermined);
    // test("a += 1n", ValueType::Undetermined);
}

#[test]
fn conditional_tests() {
    test("foo ? bar : 0", ValueType::Undetermined);
    test("foo ? 1 : 'bar'", ValueType::Undetermined); // can be number or string
    test("foo ? 1 : 0", ValueType::Number);
    test("foo ? '1' : '0'", ValueType::String);
}

#[test]
fn logical_tests() {
    test("foo && bar", ValueType::Undetermined);
    test("foo1 === foo2 && bar1 !== bar2", ValueType::Boolean);
    test("+foo && (bar1 !== bar2)", ValueType::Undetermined); // can be number or boolean

    test("foo || bar", ValueType::Undetermined);
    test("foo1 === foo2 || bar1 !== bar2", ValueType::Boolean);
    test("+foo || (bar1 !== bar2)", ValueType::Undetermined); // can be number or boolean

    test("foo ?? bar", ValueType::Undetermined);
    test("foo1 === foo2 ?? bar1 !== bar2", ValueType::Boolean);
    // test("+foo ?? (bar1 !== bar2)", ValueType::Number);
}

#[test]
fn undetermined_tests() {
    test("foo()", ValueType::Undetermined);
    test("''.foo", ValueType::Undetermined);
    test("foo.bar", ValueType::Undetermined);
    test("new foo()", ValueType::Undetermined);
}
