// @target: es6
// @experimentaldecorators: true
// @emitDecoratorMetadata: true
// @module: commonjs
// @filename: a.ts
let Testing123 = class Testing123 {
};
Testing123 = babelHelpers.decorate([
    Something({ v: () => Testing123 })
], Testing123);
export { Testing123 };
