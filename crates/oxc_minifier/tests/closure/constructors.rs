//! [OptimizeConstructor](https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/OptimizeConstructorsTest.java)
use crate::{test, test_same};

#[test]
fn test_simple() {
    // testSimple
    test(
        "
    class S { constructor() {} }
    class C extends S { constructor() { super(); } }
    let c = new C();",
        "class S{}class C extends S{}let c=new C()",
    );
    /*
        @Test
    public void testSimpleClassExpression() {
      // As simple a test case as I can come up with...
      test(
          lines(
              "const S = class { constructor() {} }",
              "let C = class extends S { constructor() { super(); } };",
              "let instance = new C();"),
          lines(
              "const S = class { };", //
              "let C = class extends S {};",
              "let instance = new C();"));
    }
       */

    // testSimpleClassExpression
    test(
        "
    class S { constructor() {} }
    class C extends S { constructor() { super(); } }
    let c = new C();",
        "class S{}class C extends S{}let c=new C()",
    );
}

#[test]
fn test_class_expression() {
    // @Test
    // public void testClassExpressionWithInComma() {
    //   // The use of the class being modified doesn't matter so
    //   // definitions in complex expressions is ok.
    //   test(
    //       lines(
    //           "class S { constructor() {} }",
    //           "let C = use ?? class extends S { constructor() { super(); } };",
    //           "let c = new C();"),
    //       lines("class S { }", "let C = use ?? class extends S { };", "let c = new C();"));
    // }

    // testClassExpressionWithInComma
    test(
        "
    class S { constructor() {} }
    let C = use ?? class extends S { constructor() { super(); } };
    let c = new C();",
        "class S{}let C=use??class extends S{},c=new C()",
    );

    // @Test
    // public void testClassExpressionDefinedWithComma() {
    //   // As simple a test case as I can come up with...
    //   test(
    //       lines(
    //           "const S = (0, class { constructor() {} });",
    //           "let C = class extends S { constructor() { super(); } };",
    //           "let instance = new C();"),
    //       lines(
    //           "const S = (0, class { });", //
    //           "let C = class extends S {};",
    //           "let instance = new C();"));
    // }

    // testClassExpressionDefinedWithComma
    test(
        "
    const S = (0, class { constructor() {} });
    let C = class extends S { constructor() { super(); } };
    let instance = new C();",
        "const S=(0,class{});let C=class extends S{},instance=new C()",
    );
}

#[test]
fn test_es5() {
    // @Test
    // public void testES5SuperClass() {
    //   // NOTE: we can remove subclasses of well defined ES5 classes
    //   test(
    //       lines(
    //           "/** @constructor */ let S = function() {};",
    //           "class C extends S { constructor() { super(); } }",
    //           "let c = new C();"),
    //       lines(
    //           "/** @constructor */ let S = function() {};",
    //           "class C extends S { }",
    //           "let c = new C();"));
    // }

    // testES5SuperClass
    test(
        "
    /** @constructor */ let S = function() {};
    class C extends S { constructor() { super(); } }
    let c = new C();",
        "let S=function(){};class C extends S{}let c=new C()",
    );

    // @Test
    // public void testES5SubClass() {
    //   test(
    //       lines(
    //           "class S { constructor() {} }",
    //           "class C extends S { constructor() { super(); } }",
    //           "function E() { return Reflect.construct(C); }",
    //           "let c = new C();"),
    //       lines(
    //           "class S { }",
    //           "class C extends S { }",
    //           "function E() { return Reflect.construct(C); }",
    //           "let c = new C();"));
    // }

    // testES5SubClass
    // FIXME: not passing
    // test(
    //     "
    // class S { constructor() {} }
    // class C extends S { constructor() { super(); } }
    // function E() { return Reflect.construct(C); }
    // let c = new C();",
    //     "class S{}class C extends S{}function E(){return Reflect.construct(C)}let c=new C()",
    // );
}

#[test]
fn test_parameter() {
    // @Test
    // public void testParameterMismatch1() {
    //   testSame(
    //       lines(
    //           "class S { constructor(a=undefined) {use(a);} }",
    //           "class C extends S { constructor() { super(); } }",
    //           "let c = new C(1);"));
    // }

    // testParameterMismatch1
    test(
        "
    class S { constructor(a=undefined) {use(a);} }
    class C extends S { constructor() { super(); } }
    let c = new C(1);",
        "class S{constructor(a=void 0){use(a)}}class C extends S{constructor(){super()}}let c = new C(1)",
    );
}
