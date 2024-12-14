let _super = function() {
  "use strict";
  babelHelpers.defineProperty(this, "bar", "foo");
  return this;
};
class Foo extends Bar {
  constructor(x = () => {
    check(_super.call(super()));
  }) {}
}
