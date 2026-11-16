for (var { a, ...b } of [{ a: 1, c: 2 }]) {}
console.log(a, b);

function f(items) {
  for (var { x, ...y } of items);
  return [x, y];
}

for (let { a1, ...b1 } of []) {}

try {} catch ({ a2, ...b2 }) {}

function g(o, list) {
  var a;
  for (var { a, ...b } in o);
  for (var [{ c, ...d }] of list);
  return [a, b, c, d];
}

function h() {}
for (var { h, ...i } of []);

async function k(items) {
  for await (var { m, ...n } of items);
  return [m, n];
}
