function outer() {
  var _this = this;
  return function() {
    class C {
      [_this]() {}
    }
  };
}

function outer2() {
  class C {
    [this]() {}
  }
}
