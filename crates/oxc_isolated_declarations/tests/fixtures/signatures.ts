export interface X {
  set value(_: string);
}

export type A = {
  set value({ a, b, c }: { a: string; b: string; c: string });
  get value();
};

export interface I {
  set value(_);
  get value(): string;
}
