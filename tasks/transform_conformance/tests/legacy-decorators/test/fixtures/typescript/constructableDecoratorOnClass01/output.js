// @experimentalDecorators: true
class CtorDtor {
}
let C = class C {
};
C = babelHelpers.decorate([
    CtorDtor
], C);
