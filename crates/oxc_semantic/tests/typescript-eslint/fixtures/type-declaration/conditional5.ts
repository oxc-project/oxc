type Test<U> = U extends (arg: { [k: string]: (arg2: infer I) => void }) => void
  ? I
  : never;
