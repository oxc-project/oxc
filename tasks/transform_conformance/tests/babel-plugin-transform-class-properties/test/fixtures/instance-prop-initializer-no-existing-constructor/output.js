class C {
  constructor() {
    babelHelpers.defineProperty(this, "function", { function: function() {} }.function);
    babelHelpers.defineProperty(this, "functions", [function() {
      function foo() {}
    }, function() {
      function foo() {}
    }]);
    babelHelpers.defineProperty(this, "arrow", { arrow: () => {} }.arrow);
    babelHelpers.defineProperty(this, "arrows", [() => () => {}, () => () => {}]);
    babelHelpers.defineProperty(this, "klass", class klass {});
    babelHelpers.defineProperty(this, "classExtends", class classExtends extends class {} {});
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
