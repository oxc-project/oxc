export interface X {
  /**
   * Comment
   */
  set value(_: string);
}

export type A = {
  /**
   * Comment
   */
  set value({ a, b, c }: { a: string; b: string; c: string });
  /**
   * Comment
   */
  get value();
};

export interface I {
  set value(_);
  get value(): string;
}


// Do nothing
export interface Ref<T = any, S = T> {
  get value(): T
  set value(_: S)
}

export interface MultipleSetterAndGetter {
  get ok(): string
  set ok(_: string)
  get bad() // infer return type
  set bad(_: string)
}
