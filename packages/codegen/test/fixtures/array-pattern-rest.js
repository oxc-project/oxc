const [foo, ...a] = arr;
const [...b] = arr;
const [, ...c] = arr;
const [, , , ...d] = arr;
const [...[e]] = arr;
const [...{ length }] = arr;
for (const [...f] of arr);

function f1([bar, ...g]) {}
function f2([...h]) {}
function f3([, ...i]) {}
function f4([, , , ...j]) {}

([bar, ...k]) => {};
([...l]) => {};
([, ...m]) => {};
([, , , ...n]) => {};

[foo, ...o] = arr;
[...p] = arr;
[, ...q] = arr;
[, , , ...r] = arr;
