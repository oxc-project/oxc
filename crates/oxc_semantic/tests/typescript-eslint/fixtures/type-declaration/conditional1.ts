type T<U> = U extends infer V ? V : never;

type Unresolved = V;
