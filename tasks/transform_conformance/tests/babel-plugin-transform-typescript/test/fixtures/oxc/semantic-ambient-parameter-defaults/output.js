function f(a = x) {
  let x = 1;
}
function nested(a = function g(b = x) {
  let x = 1;
}) {
  let x = 2;
}
const y = 0;
{
  const outer = (a = y) => {
    let y = 1;
  };
}
function unresolved(a = (() => {
  return x;
})()) {
  let x = 1;
}
function resolved(a = (() => {
  return y;
})()) {
  let y = 1;
}
function parameter(x, a = (() => {
  return x;
})()) {}
