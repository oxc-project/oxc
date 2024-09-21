function outer() {
  var _this = this;
  return function() {
    class C extends _this {}
  };
}

function outer2() {
  class C extends this {}
}
