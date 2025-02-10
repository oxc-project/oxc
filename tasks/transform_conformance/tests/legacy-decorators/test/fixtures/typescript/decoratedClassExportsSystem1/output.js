// @target: es6
// @experimentaldecorators: true
// @emitDecoratorMetadata: true
// @module: system
// @filename: a.ts
var Testing123_1;
let Testing123 = class Testing123 {
    static { Testing123_1 = this; }
    static prop0;
    static prop1 = Testing123_1.prop0;
};
Testing123 = Testing123_1 = babelHelpers.decorate([
    Something({ v: () => Testing123 })
], Testing123);
export { Testing123 };
