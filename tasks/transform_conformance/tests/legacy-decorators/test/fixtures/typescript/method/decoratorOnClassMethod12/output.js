// @target: ES5
// @experimentaldecorators: true
var M;
(function (M) {
    class S {
        decorator(target, key) { }
    }
    class C extends S {
        method() { }
    }
    babelHelpers.decorate([
        (super.decorator)
    ], C.prototype, "method", null);
})(M || (M = {}));
