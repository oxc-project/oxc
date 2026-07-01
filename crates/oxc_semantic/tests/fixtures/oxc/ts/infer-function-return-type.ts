type MyType<T> = T extends (a: infer B) => B ? [] : [];
type MyType2<T> = T extends (...a: infer B) => B ? [] : [];
type U = string;
type MyType3<T> = T extends infer U extends U ? U : [];
