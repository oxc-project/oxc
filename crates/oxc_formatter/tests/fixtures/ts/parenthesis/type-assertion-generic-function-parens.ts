// The first three statements deviate from Prettier (DIVERGENCES.md#type-assertion-generic-function-parens).
var a = < <T>(x: T) => T>f;
var b = <(<T>() => void)>f;
var c = < <T>(x: T) => T>((x) => {
  return null;
});
// No parens: no type parameters, a `new` type, or a nested function type
var d = <() => void>f;
var e = <new <T>() => T>f;
var g = <Map<string, <T>() => T>>f;
