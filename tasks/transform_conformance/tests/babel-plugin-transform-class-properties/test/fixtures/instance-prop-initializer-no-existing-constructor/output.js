class C {
  constructor() {
    babelHelpers.defineProperty(this, "function", function() {});
    babelHelpers.defineProperty(this, "functions", [function() {
      function foo() {}
    }, function() {
      function foo() {}
    }]);
    babelHelpers.defineProperty(this, "arrow", () => {});
    babelHelpers.defineProperty(this, "arrows", [() => () => {}, () => () => {}]);
    babelHelpers.defineProperty(this, "klass", class {});
    babelHelpers.defineProperty(this, "classExtends", class extends class {} {});
    babelHelpers.defineProperty(this, "classes", [class {
      method() {
        class D {}
      }
      method2() {
        class E {}
      }
    }, class {
      method() {
        class D {}
      }
      method2() {
        class E {}
      }
    }]);
  }
}
