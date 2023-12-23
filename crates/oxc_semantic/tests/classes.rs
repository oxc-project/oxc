mod util;

use util::SemanticTester;

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
    .has_number_of_methods(3)
    .has_number_of_properties(2)
    .has_method("a")
    .has_property("privateProperty");
}
