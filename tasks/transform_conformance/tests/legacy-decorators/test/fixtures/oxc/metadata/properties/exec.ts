import "reflect-metadata";

function dce() {
}

class Example {
	@dce
	count: number = 0;

	@dce
	message: string = "";
}

const example = new Example();

expect(Reflect.getMetadata("design:type", example, "count")).toBe(Number);
expect(Reflect.getMetadata("design:type", example, "message")).toBe(String);

