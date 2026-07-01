type MyType<T> = T extends (a: infer B) => B ? [] : [];
type MyType2<T> = T extends (...a: infer B) => B ? [] : [];
