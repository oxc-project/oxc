let A = class A {
};
A = babelHelpers.decorate([
    dec
], A);
let B = class B {
    constructor(x) { }
};
B = babelHelpers.decorate([
    dec,
    __metadata("design:paramtypes", [Number])
], B);
let C = class C extends A {
};
C = babelHelpers.decorate([
    dec
], C);
