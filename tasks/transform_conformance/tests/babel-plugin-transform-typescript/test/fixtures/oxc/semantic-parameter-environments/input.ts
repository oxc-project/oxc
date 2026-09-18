function erasedEnclosingBinding() {
  declare const x: number;
  function f(a = x) { let x = 1; }
}

function nestedClosure() {
  const x = 0;
  function outer(a = (() => {
    declare const x: number;
    return () => x;
  })()) { var x = 1; }
}

function nestedParameters(a = function inner(b = (() => {
  declare const x: number;
  return x;
})()) { let x = 1; }) { let x = 2; }

function catchInitializer() {
  const x = 0;
  try {} catch ({ [(() => {
    declare const x: number;
    return x;
  })()]: a }) { let x = 1; }
}

const named = function x(a = (() => {
  declare const x: number;
  return x;
})()) {};

function restParameter(a = (() => {
  declare const x: number;
  return x;
})(), ...x) {}

function enclosingBody() {
  const x = 0;
  function outer(a = (() => {
    declare const x: number;
    return x;
  })()) { let x = 1; }
}
