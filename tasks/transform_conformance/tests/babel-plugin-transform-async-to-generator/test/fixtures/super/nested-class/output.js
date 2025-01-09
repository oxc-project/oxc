class Root {}

class Outer extends Root {
  value = 0

  method() {
    var _superprop_getValue = () => super.value;
    return babelHelpers.asyncToGenerator(function* () {
      () => _superprop_getValue();

      class Inner extends Outer {
        normal() {
          console.log(super.value);
        }

        method() {
          var _superprop_getValue2 = () => super.value;
          return babelHelpers.asyncToGenerator(function* () {
            () => _superprop_getValue2();
          })();
        }
      }

      () => _superprop_getValue();
    })();
  }
}
