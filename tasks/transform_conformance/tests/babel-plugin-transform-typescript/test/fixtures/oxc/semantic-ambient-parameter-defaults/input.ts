declare const x: number;
function f(a = x) { let x = 1; }
function nested(a = function g(b = x) { let x = 1; }) { let x = 2; }
const y = 0;
{
  declare const y: number;
  const outer = (a = y) => { let y = 1; };
}
