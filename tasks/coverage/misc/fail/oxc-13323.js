o = {
  foo: function() {
    super.foo;
  },

  bar() {
    return function() {
      super.bar;
    };
  },

  [ function() { super.qux; } ]() {},

  get [ function() { super.bing; } ]() {},

  set [ function() { super.bong; } ](v) {},
};
