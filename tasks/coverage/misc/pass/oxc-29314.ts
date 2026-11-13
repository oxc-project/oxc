f<A<B>>(h);
f<typeof x>(h);
f<await>(h);
f<yield>(h);
f<let>(h);
f<A<string.x>>(h);
f<null>(h);
f<null[]>(h);
f<-1>(h);
f<1n>(h);
f<void>(h);
f<keyof T>(h);
f<unique symbol>(h);
f<readonly T[]>(h);
f<new () => T>(h);
f<abstract new () => T>(h);
f<(a: T) => T>(h);
f<<T>() => T>(h);
f<A<<T>() => T>>(h);
f<|A>(h);
f<A, B,>(h);
f<"a" | `b${c}`>(h);
new C<A<B>>(h);
f<A>`t`;
a < b > (c);
a < (b > c);
let x: A<B<C>>;
let y: A<await>;
let z: null;
class D extends E<A<B>> implements I<F> {}
async function g() { await f<A>(h); }
function* gen() { yield f<A>(h); }
// `null` is not a qualified name, so this is `(f < A) < (null.x >> h)`, as in TypeScript.
f<A<null.x>>(h);
