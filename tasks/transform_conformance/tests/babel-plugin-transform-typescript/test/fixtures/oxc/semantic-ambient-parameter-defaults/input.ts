declare const x: number;
function f(a = x) { let x = 1; }
function nested(a = function g(b = x) { let x = 1; }) { let x = 2; }
const y = 0;
{
  declare const y: number;
  const outer = (a = y) => { let y = 1; };
}
function unresolved(a = (() => { declare const x: number; return x; })()) { let x = 1; }
function resolved(a = (() => { declare const y: number; return y; })()) { let y = 1; }
function parameter(x, a = (() => { declare const x: number; return x; })()) {}
