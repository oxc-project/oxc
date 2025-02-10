class C {
    method() { }
}
class C2 {
    method(allowed) { }
}
babelHelpers.decorate([
    babelHelpers.decorateParam(0, dec)
], C2.prototype, "method", null);
