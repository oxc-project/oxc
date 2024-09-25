function outer() {
  var _this = this;
  return function() {
    class C {
      [_this] = 1;
    }
  };
}

function outer2() {
  class C {
    [this] = 1;
  }
}
