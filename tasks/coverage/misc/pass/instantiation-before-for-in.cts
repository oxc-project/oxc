const factory = null as unknown as string & { <T>(): T };
const object = {};

// Keep the instantiation before the for-in delimiter in this non-strict CommonJS file.
for (var key = factory<number> in object) {}
