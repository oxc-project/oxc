import a from "./x\uD800.ts";
import b from "./y.ts";
import "./x\uD800.mts";
import "\uD800/a.tsx";
// An extension with a lone surrogate should not be transformed.
import "./a.ts\uD800";
// Bare import should not be rewritten.
import "\uD800.ts";
console.log(a, b);
