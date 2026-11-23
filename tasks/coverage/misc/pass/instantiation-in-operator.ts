const factory = null as unknown as string & { <T>(): T };
const object = {};
type Box<T> = { value: T };

const direct = factory<number> in object;
const nested = factory<Box<number>> in object;

if (factory<string> in object) {}
for (; factory<number> in object; ) {}
for (let present = (factory<boolean> in object); present; ) {}
