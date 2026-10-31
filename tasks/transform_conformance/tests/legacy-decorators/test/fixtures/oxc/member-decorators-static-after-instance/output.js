
function dec(name) {
  return () => {};
}
class A {
  prop = 1;
  static s = 2;
  m() {}
  static sm() {}
  get g() {
    return 1;
  }
  static get sg() {
    return 1;
  }
}
babelHelpers.decorate([dec("instance-prop")], A.prototype, "prop", void 0);
babelHelpers.decorate([dec("instance-method")], A.prototype, "m", null);
babelHelpers.decorate([dec("instance-accessor")], A.prototype, "g", null);
babelHelpers.decorate([dec("static-prop")], A, "s", void 0);
babelHelpers.decorate([dec("static-method")], A, "sm", null);
babelHelpers.decorate([dec("static-accessor")], A, "sg", null);


