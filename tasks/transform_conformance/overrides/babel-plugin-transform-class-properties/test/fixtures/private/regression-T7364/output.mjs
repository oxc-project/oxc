var _myAsyncMethod2;
var _myAsyncMethod = new WeakMap();
class MyClass {
  constructor() {
    var _this = this;
    babelHelpers.classPrivateFieldInitSpec(this, _myAsyncMethod, babelHelpers.asyncToGenerator(function* () {
      console.log(_this);
    }));
  }
}
_myAsyncMethod2 = new WeakMap(), class MyClass2 {
  constructor() {
    var _this2 = this;
    babelHelpers.classPrivateFieldInitSpec(this, _myAsyncMethod2, babelHelpers.asyncToGenerator(function* () {
      console.log(_this2);
    }));
  }
};
var _myAsyncMethod3 = new WeakMap();
export default class MyClass3 {
  constructor() {
    var _this3 = this;
    babelHelpers.classPrivateFieldInitSpec(this, _myAsyncMethod3, babelHelpers.asyncToGenerator(function* () {
      console.log(_this3);
    }));
  }
}
