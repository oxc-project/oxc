import "reflect-metadata";

function dce() {}

class Example {
  count = 0;
  message = "";
}

babelHelpers.decorate([dce, babelHelpers.decorateMetadata("design:type", Number)], Example.prototype, "count", void 0);
babelHelpers.decorate([dce, babelHelpers.decorateMetadata("design:type", String)], Example.prototype, "message", void 0);

const example = new Example();

expect(Reflect.getMetadata("design:type", example, "count")).toBe(Number);
expect(Reflect.getMetadata("design:type", example, "message")).toBe(String);
