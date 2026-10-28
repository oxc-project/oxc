interface Source {
  value: string;
}

export function mapped(
  value: { [K in keyof Source]: Source[K] } = { value: "" },
  required: string,
): void {}

export function keyOf(
  value: keyof Source = "value",
  required: string,
): void {}

export function readonlyTuple(
  value: readonly [string] = [""],
  required: string,
): void {}

export function intersection(
  value: { optional?: string } & number[] = [],
  required: string,
): void {}

export function operatorIntersection(
  value: (keyof Source) & string = "value",
  required: string,
): void {}

export function unknownOperatorIntersection(
  value: unknown & keyof Source = "value",
  required: string,
): void {}

export function unresolvedIntersection<T>(
  value: T & string = "" as T & string,
  required: string,
): void {}

export function allUnknownIntersection(
  value: unknown & unknown = undefined,
  required: string,
): void {}

export function unresolvedUndefinedUnionIntersection<T>(
  value: (T | undefined) & string = "" as (T | undefined) & string,
  required: string,
): void {}

export function unknownUndefinedUnionOperatorIntersection(
  value: (unknown | undefined) & keyof Source = "value",
  required: string,
): void {}

export function anyUndefinedUnionOperatorIntersection(
  value: (any | undefined) & keyof Source = "value",
  required: string,
): void {}

export function unresolvedNeverOperatorIntersection<T>(
  value: T & never & keyof Source = undefined as never,
  required: string,
): void {}

export function voidUnionUndefinedIntersection(
  value: (void | string) & undefined = undefined,
  required: string,
): void {}
