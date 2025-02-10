// @target: es5
// @experimentaldecorators: true
// @emitDecoratorMetadata: true
// @filename: a.ts
var Foo_1, Foo_2;
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
try {
    let Foo = Foo_2 = class Foo {
        static func() {
            return new Foo_2();
        }
    };
    Foo = Foo_2 = babelHelpers.decorate([
        decorator()
    ], Foo);
    Foo.func();
}
catch (e) { }
