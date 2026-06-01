// Local interface shadows the global `ReadonlyArray` — expect `Object`, not `Array`.

interface ReadonlyArray<T> { custom: T }

declare function dec(target: any, key: string): void;

class Source {
  @dec value!: ReadonlyArray<string>;
}
