function foo() {
  var _this = this;
  {
    let f = function() {
      return _this;
    };
  }
  {
    let f2 = function() {
      return _this;
    };
  }
}
