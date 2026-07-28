// Cross-file imported enum: the guard returns the actual enum object at runtime,
// so consumers (type-graphql, typeorm, NestJS Swagger, AutoMapper, class-validator)
// can introspect members via `Object.values()`. See oxc-project/oxc#14740.

import { StringEnum } from './enums';

declare function dec(target: any, key: string): void;

class Source {
  @dec value!: StringEnum;
}
