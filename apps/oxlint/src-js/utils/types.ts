/**
 * Utility type to make specified properties of a type nullable.
 *
 * @example
 * ```ts
 * type Foo = { str: string, num: number };
 * type FooWithNullableNum = SetNullable<Foo, 'num'>;
 * // { str: string, num: number | null }
 * ```
 */
export type SetNullable<BaseType, Keys extends keyof BaseType = keyof BaseType> = {
  [Key in keyof BaseType]: Key extends Keys ? BaseType[Key] | null : BaseType[Key];
};
