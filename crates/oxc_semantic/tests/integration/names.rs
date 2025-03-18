use crate::util::SemanticTester;

#[test]
fn test_declarations() {
    SemanticTester::js(
        "
        function foo() {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        class Foo {}
      ",
    )
    .has_symbol("Foo")
    .is_symbol_for_class_name()
    .test();
}

#[test]
fn test_simple_declare_init() {
    SemanticTester::js(
        "
        var foo = function() {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var foo = () => {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var Foo = class {}
      ",
    )
    .has_symbol("Foo")
    .is_symbol_for_class_name()
    .test();
}

#[test]
fn test_simple_assign() {
    SemanticTester::js(
        "
        var foo;
        foo = function() {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var foo;
        foo = () => {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var Foo;
        Foo = class {}
      ",
    )
    .has_symbol("Foo")
    .is_symbol_for_class_name()
    .test();

    SemanticTester::js(
        "
        var foo;
        foo ||= function() {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var foo = 1;
        foo &&= function() {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var foo;
        foo ??= function() {}
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();
}

#[test]
fn test_default_declaration() {
    SemanticTester::js(
        "
        var [foo = function() {}] = [];
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var [foo = () => {}] = [];
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var [Foo = class {}] = [];
      ",
    )
    .has_symbol("Foo")
    .is_symbol_for_class_name()
    .test();

    SemanticTester::js(
        "
        var { foo = function() {} } = {};
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();
}

#[test]
fn test_default_assign() {
    SemanticTester::js(
        "
        var foo;
        [foo = function() {}] = [];
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var foo;
        [foo = () => {}] = [];
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        var Foo;
        [Foo = class {}] = [];
      ",
    )
    .has_symbol("Foo")
    .is_symbol_for_class_name()
    .test();

    SemanticTester::js(
        "
        var foo;
        ({ foo = function() {} } = {});
      ",
    )
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();
}

/// This is Annex B feature.
#[test]
fn test_for_in_declaration() {
    SemanticTester::js(
        "
        for (var foo = function() {} in []) {}
      ",
    )
    .expect_errors(true)
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        for (var foo = () => {} in []) {}
      ",
    )
    .expect_errors(true)
    .has_symbol("foo")
    .is_symbol_for_function_name()
    .test();

    SemanticTester::js(
        "
        for (var Foo = class {} in []) {}
      ",
    )
    .expect_errors(true)
    .has_symbol("Foo")
    .is_symbol_for_class_name()
    .test();
}
