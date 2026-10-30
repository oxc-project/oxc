// The first seven statements deviate from Prettier (DIVERGENCES.md#generic-function-type-after-lt-parens).
var a = < <T>(x: T) => T>f;
var b = <(<T>() => void)>f;
var c = < <T>(x: T) => T>((x) => {
  return null;
});
class A extends B< <T>() => T> {}
class C implements D< <T>() => T> {}
interface E extends F< <T>() => T> {}
type G = typeof g< <T>() => T>;
// No parens: no type parameters, a `new` type, a nested function type, not the first type argument, or a `<` TypeScript splits
var d = <() => void>f;
var e = <new <T>() => T>f;
var g = <Map<string, <T>() => T>>f;
class H extends B<X, <T>() => T> {}
type I = Array< <T>() => T>;
