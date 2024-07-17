type Test<U> = U extends { [k: string]: infer I } ? I : never;
