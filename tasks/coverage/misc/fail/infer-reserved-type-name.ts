type T<X> = X extends infer string ? string : never;
