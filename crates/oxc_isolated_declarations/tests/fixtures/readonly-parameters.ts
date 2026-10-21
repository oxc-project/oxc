export class ReadonlyArrayProperty {
  constructor(readonly value?: readonly string[]) {}
}

export class ReadonlyTupleProperty {
  constructor(public value?: readonly [string, number?]) {}
}

export class GenericReadonlyProperty<T> {
  constructor(readonly value?: readonly (T | undefined)[]) {}
}

export class ParenthesizedReadonlyProperty {
  constructor(readonly value?: (readonly string[])) {}
}

export class ReadonlyUnionProperty {
  constructor(readonly value?: string | readonly string[]) {}
}

export class ReadonlyIntersectionProperty {
  constructor(readonly value?: (readonly string[]) & { length: number }) {}
}

export class ExplicitUndefinedReadonlyProperty {
  constructor(readonly value?: readonly string[] | undefined) {}
}

export class DefaultReadonlyProperty {
  constructor(readonly value: readonly string[] = [], required: string) {}
}

export function defaultReadonlyArray(
  value: readonly string[] = [],
  required: string,
): void {}

export function defaultReadonlyTuple(
  value: readonly [string] = [""],
  required: string,
): void {}

export function defaultGenericReadonlyArray<T>(
  value: readonly T[] = [],
  required: string,
): void {}

export function trailingReadonlyDefault(value: readonly string[] = []): void {}

export function optionalReadonlyArray(value?: readonly string[]): void {}

export function readonlyRest(...values: readonly string[]): void {}
