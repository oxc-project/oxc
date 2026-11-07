function f({ a, ...x }, b = a) { return b; }
const arrow = ({ a, ...rest }, b = a) => [b, rest];
function ordered({ a, ...rest }, b = a, { c } = { c: b }, ...tail) { return [c, tail]; }
function forward(b = a, { a, ...rest }) { return b; }
function nested({ a, ...rest }, { b = a }) { return b; }
function computed({ a, ...rest }, { [a]: b }) { return b; }
function closure({ a, ...rest }, selected = a, read = () => a) { return read(); }
const outside = 7;
function shadowed({ a, ...rest }, b = a, read = () => outside) { let outside = 9; return read(); }
function redeclared({ a, ...rest }, selected = a, read = () => a) { var a = 2; return [a, read()]; }
function* callTime({ a, ...rest }, selected = a, read = () => a) { yield read(); }
function bodyEval({ a, ...rest }, b = a) { let hidden = 3; return eval('hidden') + b; }
async function shadowAsync({ a, ...rest }, selected = a, read = () => a, other = () => outside) { let outside = 2; return 3; }
function* argumentIdentity({ a, ...rest }, b = a, saved = arguments) { yield saved === arguments; }
