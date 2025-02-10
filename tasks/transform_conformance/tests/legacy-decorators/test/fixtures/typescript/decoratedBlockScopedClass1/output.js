// @target: es5
// @experimentaldecorators: true
// @emitDecoratorMetadata: true
// @filename: a.ts
var Foo_1;
function decorator() {
    return (target) => { };
}
let Foo = Foo_1 = class Foo {
    static func() {
        return new Foo_1();
    }
};
Foo = Foo_1 = babelHelpers.decorate([
    decorator()
], Foo);
Foo.func();
