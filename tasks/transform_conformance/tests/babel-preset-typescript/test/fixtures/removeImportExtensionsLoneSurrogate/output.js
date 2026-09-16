import a from "./x\uD800";
import b from "./y";
import "./x\uD800";
import "\uD800/a";
// An extension with a lone surrogate should not be transformed.
import "./a.ts\uD800";
// Bare import should not be rewritten.
import "\uD800.ts";
console.log(a, b);
