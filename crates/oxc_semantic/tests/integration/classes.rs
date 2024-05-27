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
