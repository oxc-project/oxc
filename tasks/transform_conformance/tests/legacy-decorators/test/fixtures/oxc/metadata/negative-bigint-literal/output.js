import "reflect-metadata";

function dce() {}

class Example {
  negative;
  positive;
  number;
}

babelHelpers.decorate([dce, babelHelpers.decorateMetadata("design:type", BigInt)], Example.prototype, "negative", void 0);
babelHelpers.decorate([dce, babelHelpers.decorateMetadata("design:type", BigInt)], Example.prototype, "positive", void 0);
babelHelpers.decorate([dce, babelHelpers.decorateMetadata("design:type", Number)], Example.prototype, "number", void 0);

const example = new Example();

expect(Reflect.getMetadata("design:type", example, "negative")).toBe(BigInt);
expect(Reflect.getMetadata("design:type", example, "positive")).toBe(BigInt);
expect(Reflect.getMetadata("design:type", example, "number")).toBe(Number);
