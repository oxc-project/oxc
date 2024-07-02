type Test<U> = U extends (k: infer I) => void ? I : never;
