// Ambient `declare class X {}`: the guard falls back to `Object` when the
// runtime binding is missing, avoiding a `ReferenceError` for stale ambient
// declarations and returning the actual class when the runtime provides it.

declare class Ambient {}

declare function dec(target: any, key: string): void;

class Source {
  @dec value!: Ambient;
}
