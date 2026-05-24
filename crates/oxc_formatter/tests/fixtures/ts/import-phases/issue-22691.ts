import defer * as b from "./b";

console.log(b.b);

import source wasm from "./mod.wasm";

import defer * as c from "./c" with { type: "json" };
