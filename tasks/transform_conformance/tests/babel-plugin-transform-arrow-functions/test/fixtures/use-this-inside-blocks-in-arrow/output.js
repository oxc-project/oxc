function outer() {
  var _this = this;

  let f = function() {
    {
      let t = _this;
    }
  };

  let f2 = function() {
    if (x) {
      if (y) {
        return _this;
      }
    }
  };
}
