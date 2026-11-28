import "reflect-metadata";

function dce() {
}

class Example {
	@dce
	negative: -1n;

	@dce
	positive: 1n;

	@dce
	number: -1;
}

const example = new Example();

expect(Reflect.getMetadata("design:type", example, "negative")).toBe(BigInt);
expect(Reflect.getMetadata("design:type", example, "positive")).toBe(BigInt);
expect(Reflect.getMetadata("design:type", example, "number")).toBe(Number);
