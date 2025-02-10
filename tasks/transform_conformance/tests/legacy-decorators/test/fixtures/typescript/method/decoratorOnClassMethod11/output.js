// @target: ES5
// @experimentaldecorators: true
var M;
(function (M) {
    class C {
        decorator(target, key) { }
        method() { }
    }
    babelHelpers.decorate([
        (this.decorator)
    ], C.prototype, "method", null);
})(M || (M = {}));
