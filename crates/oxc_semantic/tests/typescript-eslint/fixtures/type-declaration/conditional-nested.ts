type Test<T> =
  T extends Array<infer U> ? U : T extends Set<infer U> ? U : never;
