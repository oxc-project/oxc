let a;
// the default param value is resolved to the outer scope
function foo(b = a) {
  let a;
}
