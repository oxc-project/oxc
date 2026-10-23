export interface S {
  a: number;
}

export type Alias = S;

export type MaybeS = S | undefined;

export function reference(s: S = { a: 1 }, w: string): void {}

export function alias(s: Alias = { a: 1 }, w: string): void {}

export function generic(s: Array<S> = [], w: string): void {}

export function typeParameter<T>(value: T = undefined as T, w: string): void {}

export function nullable(s: S | null = null, w: string): void {}

// Excluded: `any | S` is `any`, which already includes `undefined`.
export function anyUnion(s: any | S = 1, w: string): void {}

// Already includes `undefined`; adding it again is redundant but harmless.
export function aliasWithUndefined(s: MaybeS = undefined, w: string): void {}

export function explicitUndefined(s: S | undefined = undefined, w: string): void {}

export function lastParameter(w: string, s: S = { a: 1 }): void {}

// Deeply nested unions are walked once per level.
export function nestedUnion(s: ((((((((((((((((((((((((((((((((S | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) | string) = { a: 1 }, w: string): void {}
