// @target: es2015
// @experimentaldecorators: true
function fn(value) {
    class Class {
        async method(arg) { }
    }
    babelHelpers.decorate([
        babelHelpers.decorateParam(0, dec(await value))
    ], Class.prototype, "method", null);
    return Class;
}
