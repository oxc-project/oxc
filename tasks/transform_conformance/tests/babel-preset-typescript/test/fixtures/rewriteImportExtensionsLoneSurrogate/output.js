import a from "./x\uD800.js";
import b from "./y.js";
import "./x\uD800.mjs";
import "\uD800/a.js";
// An extension with a lone surrogate should not be transformed.
import "./a.ts\uD800";
// Bare import should not be rewritten.
import "\uD800.ts";
console.log(a, b);
