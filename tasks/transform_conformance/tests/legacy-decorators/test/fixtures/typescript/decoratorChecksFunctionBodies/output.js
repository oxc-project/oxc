// @target:es5
// @experimentaldecorators: true
// from #2971
function func(s) {
}
class A {
    m() {
    }
}
babelHelpers.decorate([
    ((x, p, d) => {
        var a = 3;
        func(a);
        return d;
    })
], A.prototype, "m", null);
