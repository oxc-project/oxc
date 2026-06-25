use crate::{test, test_same};

#[test]
fn test_remove_unused_private_fields() {
    test(
        "class C { #unused = 1; #used = 2; method() { return this.#used; } } new C();",
        "class C { #used = 2; method() { return this.#used; } } new C();",
    );
    test(
        "class C { #unused = 1; #used = 2; method(foo) { return #used in foo; } } new C();",
        "class C { #used = 2; method(foo) { return #used in foo; } } new C();",
    );
    test(
        "class C { #unused; #used; method() { return this.#used; } } new C();",
        "class C { #used; method() { return this.#used; } } new C();",
    );
    test("class C { #a = 1; #b = 2; #c = 3; } new C();", "class C { } new C();");
    test(
        "class C { static #unused = 1; static #used = 2; static method() { return C.#used; } }",
        "class C { static #used = 2; static method() { return C.#used; } }",
    );
    test(
        "class C { public = 1; #unused = 2; #used = 3; method() { return this.public + this.#used; } } new C();",
        "class C { public = 1; #used = 3; method() { return this.public + this.#used; } } new C();",
    );
    test_same("class C { #unused = foo(); method() { return 1; } } new C();");
    test_same("class C { #used = 1; method() { return eval('this.#used'); } } new C();");
}

#[test]
fn test_remove_unused_private_methods() {
    test(
        "class C { #unusedMethod() { return 1; } #usedMethod() { return 2; } method() { return this.#usedMethod(); } } new C();",
        "class C { #usedMethod() { return 2; } method() { return this.#usedMethod(); } } new C();",
    );
    test("class C { #a() {} #b() {} #c() {} } new C();", "class C { } new C();");
    test(
        "class C { static #unusedMethod() { return 1; } static #usedMethod() { return 2; } static method() { return C.#usedMethod(); } }",
        "class C { static #usedMethod() { return 2; } static method() { return C.#usedMethod(); } }",
    );
    test_same("class C { #helper() { return 1; } method() { return this.#helper(); } } new C();");
    test_same(
        "class C { #helper() { return 1; } method() { return eval('this.#helper()'); } } new C();",
    );
}

#[test]
fn test_remove_unused_private_accessors() {
    test(
        "class C { accessor #unused = 1; accessor #used = 2; method() { return this.#used; } } new C();",
        "class C { accessor #used = 2; method() { return this.#used; } } new C();",
    );
    test_same("class C { accessor #unused = foo(); method() { return 1; } } new C();");
}

#[test]
fn test_nested_classes() {
    test(
        r"class Outer {
            #shared = 1;
            #unusedOuter = 2;

            method() {
                return this.#shared;
            }

            getInner() {
                return class Inner {
                    #shared = 3;
                    #unusedInner = 4;

                    method() {
                        return this.#shared;
                    }
                };
            }
        } new Outer();",
        r"class Outer {
            #shared = 1;

            method() {
                return this.#shared;
            }

            getInner() {
                return class {
                    #shared = 3;

                    method() {
                        return this.#shared;
                    }
                };
            }
        } new Outer();",
    );
    test_same(
        r"class Outer {
            #shared = 1;

            getInner() {
                let self = this;
                return class {
                    method() {
                        return self.#shared;
                    }
                };
            }
        } new Outer();",
    );
}
