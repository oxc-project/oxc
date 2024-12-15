class C {
  function = function() {};
  functions = [
    function() {
      function foo() {}
    },
    function() {
      function foo() {}
    }
  ];
  arrow = () => {};
  arrows = [() => () => {}, () => () => {}];
  klass = class {};
  classExtends = class extends class {} {};
  classes = [
    class {
      method() {
        class D {}
      }
      method2() {
        class E {}
      }
    },
    class {
      method() {
        class D {}
      }
      method2() {
        class E {}
      }
    }
  ];
}
