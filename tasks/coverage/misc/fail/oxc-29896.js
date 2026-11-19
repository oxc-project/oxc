// https://github.com/oxc-project/oxc/issues/29896
// `await` in the parameters of a non-async function, and in class field
// initializers, is not top-level await: it is an error in every module kind.
function fn(x = await 1) {}
var fe = function (x = await 1) {};
function* g(x = await 1) {};
({ m(x = await 1) {} });
class C { m(x = await 1) {} }
class D { p = await 0 }
class D2 { static p = await 0 }
class D3 { accessor p = await 0 }
