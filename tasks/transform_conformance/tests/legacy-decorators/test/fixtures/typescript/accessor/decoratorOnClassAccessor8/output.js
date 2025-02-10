class A {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec,
    __metadata("design:type", Number),
    __metadata("design:paramtypes", [Number])
], A.prototype, "x", null);
class B {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec,
    __metadata("design:type", Number),
    __metadata("design:paramtypes", [Number])
], B.prototype, "x", null);
class C {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec,
    __metadata("design:type", Number),
    __metadata("design:paramtypes", [Number])
], C.prototype, "x", null);
class D {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec,
    __metadata("design:type", Number),
    __metadata("design:paramtypes", [Number])
], D.prototype, "x", null);
class E {
    get x() { return 0; }
}
babelHelpers.decorate([
    dec,
    __metadata("design:type", Object),
    __metadata("design:paramtypes", [])
], E.prototype, "x", null);
class F {
    set x(value) { }
}
babelHelpers.decorate([
    dec,
    __metadata("design:type", Number),
    __metadata("design:paramtypes", [Number])
], F.prototype, "x", null);
