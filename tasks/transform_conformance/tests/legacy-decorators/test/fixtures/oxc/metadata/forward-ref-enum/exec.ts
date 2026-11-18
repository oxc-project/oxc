import "reflect-metadata";

function dce() {}

class Example {
  @dce
  str: StringEnum;

  @dce
  num: NumberEnum;

  @dce
  mixed: MixedEnum;

  @dce
  merged: MergedEnum;
}

enum StringEnum {
  A = "a",
}

enum NumberEnum {
  A,
}

enum MixedEnum {
  A = "a",
  B = 1,
}

// Merged enum with a string member and a number member, so `Object`.
enum MergedEnum {
  A = "a",
}

enum MergedEnum {
  B = 1,
}

const example = new Example();

expect(Reflect.getMetadata("design:type", example, "str")).toBe(String);
expect(Reflect.getMetadata("design:type", example, "num")).toBe(Number);
expect(Reflect.getMetadata("design:type", example, "mixed")).toBe(Object);
expect(Reflect.getMetadata("design:type", example, "merged")).toBe(Object);
