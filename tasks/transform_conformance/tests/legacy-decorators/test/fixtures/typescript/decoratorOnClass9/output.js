var B_1;
class A {
}
// https://github.com/Microsoft/TypeScript/issues/16417
let B = class B extends A {
    static { B_1 = this; }
    static x = 1;
    static y = B_1.x;
    m() {
        return B_1.x;
    }
};
B = B_1 = babelHelpers.decorate([
    dec
], B);
