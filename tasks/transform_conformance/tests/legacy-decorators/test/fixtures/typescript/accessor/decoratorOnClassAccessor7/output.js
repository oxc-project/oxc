class A {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec1
], A.prototype, "x", null);
class B {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec2
], B.prototype, "x", null);
class C {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec1
], C.prototype, "x", null);
class D {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec2
], D.prototype, "x", null);
class E {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec1
], E.prototype, "x", null);
class F {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec1
], F.prototype, "x", null);
