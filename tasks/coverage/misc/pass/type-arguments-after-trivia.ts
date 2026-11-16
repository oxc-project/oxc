// Each call has no arguments, so reading its `<` as a comparison is a syntax error.
function f<A = unknown, B = unknown>(): void {}
type T = 1;
type É = 1;

f<T /* comment */>();
f<T // comment
>();
f< /* comment */ T>();
f<a /* comment */ .b.c>();
f<a. /* comment */ b.c>();
f<a.b.c /* comment */ , T>();
f<T /* comment */ extends 1 ? 2 : 3>();
f<T? /* comment */ >();
f<T>();
f<T\u{54}>();
f<É>();
f<a.b.c>();
f<1e-5>();
f<1_000>();
f<0x1f>();
f<.5>();
f<1n>();
f<"s">();
f<this>();
f<null>();
f<T | 1>();
f<T & 1>();
f<T[]>();
f<T["x"]>();
f<T!>();
f<T extends 1 ? 2 : 3>();
f<<U>() => U>();
f<</* comment */ U>() => U>();
f<<U /* comment */ >() => U>();
f<<U>() => U>();
f<<const U>() => U>();
f<<U extends 1>() => U>();
f<<U = 1>() => U>();
f<<U, V>() => U>();
