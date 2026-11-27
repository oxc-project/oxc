// Destructuring with an object rest moved from the loop head / catch param into the body.
for (const { a1 = () => {}, ...r1 } of []) {}
for (let { a2 = function () {}, ...r2 } in {}) {}
for (var { a3 = () => {}, ...r3 } of []) {}
for (var { a4 = () => {}, ...r4 } in {}) {}
function f() {
  for (var { a5 = () => {}, ...r5 } of []) {}
}

var a, x;
for ({ a = () => {}, ...x } of []) {}
for ({ a = function () {}, ...x } in {}) {}
for ([{ a = () => {}, ...x }] of []) a;

try {} catch ({ a6 = () => {}, ...r6 }) {}
try {} catch ({ a7: { b7 = () => {}, ...r7 }, c7 = function () {} }) {}

// Nested patterns: only the object pattern with the rest moves; `c`'s default stays in the head.
for (const [{ a8 = () => {}, ...r8 }, c8 = () => {}] of [() => {}]) {}
for (const { a9: { b9 = () => {}, ...r9 }, c9 = () => {} } of []) {}

// Non-block bodies are wrapped in a new block.
for (const { ...r10 } of []) (() => {});
for ({ ...x } of []) (() => {});
