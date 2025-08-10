use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, Statement};
use oxc_ecmascript::{
    GlobalContext,
    constant_evaluation::{DetermineValueType, ValueType},
};
use oxc_parser::Parser;
use oxc_span::SourceType;

struct GlobalReferenceChecker {
    global_variable_names: Vec<String>,
}

impl<'a> GlobalContext<'a> for GlobalReferenceChecker {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> Option<bool> {
        Some(self.global_variable_names.iter().any(|name| name == ident.name.as_str()))
    }
}

fn test(source_text: &str, expected: ValueType) {
    test_with_global_variables(
        source_text,
        vec!["undefined".to_string(), "NaN".to_string(), "Infinity".to_string()],
        expected,
    );
}

fn test_with_global_variables(
    source_text: &str,
    global_variable_names: Vec<String>,
    expected: ValueType,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let global_reference_checker = GlobalReferenceChecker { global_variable_names };

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    let result = stmt.expression.value_type(&global_reference_checker);
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
    test("(() => {})", ValueType::Object);
    test("(class {})", ValueType::Object);
    test("import.meta", ValueType::Object);
    // NOTE: new.target is undefined or object
}

#[test]
fn identifier_tests() {
    test("undefined", ValueType::Undefined);
    test_with_global_variables("undefined", vec![], ValueType::Undetermined);
    test("NaN", ValueType::Number);
    test_with_global_variables("NaN", vec![], ValueType::Undetermined);
    test("Infinity", ValueType::Number);
    test_with_global_variables("Infinity", vec![], ValueType::Undetermined);
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
    test("~0", ValueType::Number);
    test("~0n", ValueType::BigInt);
    test("~foo", ValueType::Undetermined); // can be number or bigint

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
    test("undefined + true", ValueType::Number);
    test("null + true", ValueType::Number);
    test("0 + true", ValueType::Number);
    test("true + 0n", ValueType::Undetermined); // throws an error
    test("({} + [])", ValueType::String); // [object Object]
    test("[] + {}", ValueType::String); // [object Object]

    test("1 - 0", ValueType::Number);
    test("1n - 0n", ValueType::BigInt);
    test("1 - 0n", ValueType::Number); // throws an error
    test("foo - 1", ValueType::Number);
    test("foo - 1n", ValueType::BigInt);
    test("foo - undefined", ValueType::Number);
    test("foo - null", ValueType::Number);
    test("foo - ''", ValueType::Number);
    test("foo - true", ValueType::Number);
    test("foo - bar", ValueType::Undetermined); // number or bigint
    test("1 * 0", ValueType::Number);
    test("1 / 0", ValueType::Number);
    test("1 % 0", ValueType::Number);
    test("1 << 0", ValueType::Number);
    test("1 | 0", ValueType::Number);
    test("1 >> 0", ValueType::Number);
    test("1 ^ 0", ValueType::Number);
    test("1 & 0", ValueType::Number);
    test("1 ** 0", ValueType::Number);

    test("1 >>> 0", ValueType::Number);
    test("1n >>> 0n", ValueType::Number); // throws an error

    // foo * bar can be number, but the result is always a bigint (if no error happened)
    test("foo * bar * 1n", ValueType::BigInt);
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
    test("#foo in bar", ValueType::Boolean);
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

    test("a += ''", ValueType::String);
    // if `a` is a string, the result is string
    // if `a` is an object, the result can be string or number or bigint
    // if `a` is not a string nor an object, the result can be number or bigint
    test("a += 1", ValueType::Undetermined);
    test("a += 1n", ValueType::Undetermined);

    test("a -= 1", ValueType::Number); // an error is thrown if `a` is converted to bigint
    test("a -= undefined", ValueType::Number);
    test("a -= null", ValueType::Number);
    test("a -= ''", ValueType::Number);
    test("a -= true", ValueType::Number);
    test("a -= 1n", ValueType::BigInt); // an error is thrown if `a` is converted to number
    test("a -= {}", ValueType::Undetermined); // number or bigint
    test("a *= 1", ValueType::Number);
    test("a /= 1", ValueType::Number);
    test("a %= 1", ValueType::Number);
    test("a <<= 1", ValueType::Number);
    test("a |= 1", ValueType::Number);
    test("a >>= 1", ValueType::Number);
    test("a ^= 1", ValueType::Number);
    test("a &= 1", ValueType::Number);
    test("a **= 1", ValueType::Number);

    test("a >>>= 1", ValueType::Number);
    test("a >>>= 1n", ValueType::Number); // throws an error
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
    test("+foo ?? (bar1 !== bar2)", ValueType::Number);
    test("(void foo) ?? (bar1 !== bar2)", ValueType::Boolean);
}

#[test]
fn undetermined_tests() {
    test("foo++", ValueType::Undetermined); // can be number or bigint
    test("foo--", ValueType::Undetermined); // can be number or bigint

    test("foo ||= 1", ValueType::Undetermined);
    test("foo &&= 1", ValueType::Undetermined);
    test("foo ??= 1", ValueType::Undetermined);

    test("this", ValueType::Undetermined);
    test("foo()", ValueType::Undetermined);
    test("''.foo", ValueType::Undetermined);
    test("foo.bar", ValueType::Undetermined);
    test("new foo()", ValueType::Undetermined);
}
