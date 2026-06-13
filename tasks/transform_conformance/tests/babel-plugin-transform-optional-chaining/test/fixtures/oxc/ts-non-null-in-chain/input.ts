// Panicked when transforming optional chaining with TS non-null assertion inside a computed key.
// https://github.com/oxc-project/oxc/issues/22246
const x = a[b?.c!]?.["d"];
const y = a?.[b?.c!]?.d;
const z = a[b?.c!.d!]?.e;
