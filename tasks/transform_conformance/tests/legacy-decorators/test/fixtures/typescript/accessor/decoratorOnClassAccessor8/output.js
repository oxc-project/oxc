class A {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Number),
    babelHelpers.decorateMetadata("design:paramtypes", [Number])
], A.prototype, "x", null);
class B {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Number),
    babelHelpers.decorateMetadata("design:paramtypes", [Number])
], B.prototype, "x", null);
class C {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Number),
    babelHelpers.decorateMetadata("design:paramtypes", [Number])
], C.prototype, "x", null);
class D {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Number),
    babelHelpers.decorateMetadata("design:paramtypes", [Number])
], D.prototype, "x", null);
class E {
    get x() { return 0; }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Object),
    babelHelpers.decorateMetadata("design:paramtypes", [])
], E.prototype, "x", null);
class F {
    set x(value) { }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Number),
    babelHelpers.decorateMetadata("design:paramtypes", [Number])
], F.prototype, "x", null);
