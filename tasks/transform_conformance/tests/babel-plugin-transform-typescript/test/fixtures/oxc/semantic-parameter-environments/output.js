function erasedEnclosingBinding() {
  function f(a = x) { let x = 1; }
}

function nestedClosure() {
  const x = 0;
  function outer(a = (() => {
    return () => x;
  })()) { var x = 1; }
}

function nestedParameters(a = function inner(b = (() => {
  return x;
})()) { let x = 1; }) { let x = 2; }

function catchInitializer() {
  const x = 0;
  try {} catch ({ [(() => {
    return x;
  })()]: a }) { let x = 1; }
}

const named = function x(a = (() => {
  return x;
})()) {};

function restParameter(a = (() => {
  return x;
})(), ...x) {}

function enclosingBody() {
  const x = 0;
  function outer(a = (() => {
    return x;
  })()) { let x = 1; }
}
