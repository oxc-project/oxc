use crate::util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js(
        "
      class Foo {
        #privateProperty = 1;
        publicProperty = 2;

        constructor() {} // this method is skip
        a() {}
        set b(v) {}
        get b() {}
      }

    ",
    )
    .has_class("Foo")
    .has_number_of_elements(5)
    .has_method("a")
    .has_property("privateProperty");
}

#[test]
fn test_class_with_ts() {
    SemanticTester::ts(
        "
      class Foo {
        accessor ap = 1;
        accessor #pap = 1;
        constructor() {} // this method is skip
      }

    ",
    )
    .has_class("Foo")
    .has_number_of_elements(2)
    .has_accessor("ap")
    .has_accessor("pap");
}

fn check_diagnostics(source: &str) -> Vec<String> {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "Parse errors: {:?}", parsed.diagnostics);
    let ret =
        oxc_semantic::SemanticBuilder::new().with_check_syntax_error(true).build(&parsed.program);
    ret.diagnostics.errors().map(oxc_diagnostics::OxcDiagnostic::render).collect()
}

#[test]
fn checker_overload_implementation_missing() {
    // A lone private overload has no implementation (tsc TS2391).
    let diags = check_diagnostics("class C { #foo(): void; }");
    assert!(
        diags.iter().any(|m| m.contains("implementation is missing")),
        "expected a missing-implementation diagnostic, got {diags:?}"
    );

    // A private overload followed by an implementation with a different private name.
    let diags = check_diagnostics("class C { #foo(): void; #bar() {} }");
    assert!(
        diags.iter().any(|m| m.contains("implementation is missing")),
        "expected a missing-implementation diagnostic, got {diags:?}"
    );

    // Two well-known-symbol keys are provably different names (tsc TS2389).
    let diags = check_diagnostics("class C { [Symbol.iterator](); [Symbol.asyncIterator]() {} }");
    assert!(
        diags.iter().any(|m| m.contains("implementation is missing")),
        "expected a missing-implementation diagnostic, got {diags:?}"
    );

    // A well-known-symbol overload followed by a differently named implementation.
    let diags = check_diagnostics("class C { [Symbol.iterator](); foo() {} }");
    assert!(
        diags.iter().any(|m| m.contains("implementation is missing")),
        "expected a missing-implementation diagnostic, got {diags:?}"
    );

    // Consecutive overloads without an implementation are all reported.
    let diags = check_diagnostics("class C { foo(): void; bar(): void; }");
    assert_eq!(
        diags.iter().filter(|m| m.contains("implementation is missing")).count(),
        2,
        "expected two missing-implementation diagnostics, got {diags:?}"
    );

    // Clean shapes stay clean.
    let diags = check_diagnostics("class C { foo(): void; foo() {} }");
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");

    let diags = check_diagnostics("class C { [Symbol.iterator](); [Symbol.iterator]() {} }");
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");

    let diags = check_diagnostics("class C { #foo(): void; #foo() {} }");
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");

    // Unknown computed keys must not gain diagnostics.
    let diags =
        check_diagnostics("declare function foo(): string; class C { [foo()](); foo() {} }");
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");

    // Ambient classes are exempt.
    let diags = check_diagnostics("declare class C { foo(): void; }");
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
}
