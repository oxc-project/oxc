type X = string | number;
type Id<T> = T extends { id: infer Id extends X } ? Id : never;
