// Cross-file imported enum: `StringEnum` is a value-import binding. Previously
// OXC wrapped this in `typeof X === "function" ? X : Object`. Because enums
// lower to plain objects (`{ A: 'a', B: 'b' }`), the `typeof === "function"`
// test is always false and the metadata silently degraded to `Object`.
//
// The patched emit produces a bare identifier, matching babel. At runtime
// `Reflect.getMetadata('design:type', ...)` returns the enum object itself,
// which downstream consumers (NestJS Swagger, AutoMapper) introspect via
// `Object.values(t)`.

import { StringEnum } from './enums';

declare function dec(target: any, key: string): void;

class Source {
  @dec value!: StringEnum;
}
