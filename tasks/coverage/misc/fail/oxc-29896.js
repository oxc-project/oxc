function fn(x = await 1) {}
var fe = function (x = await 1) {};
function* g(x = await 1) {}
({ m(x = await 1) {} });
class C { m(x = await 1) {} }
class D { p = await 0 }
class E { static p = await 0 }
class F { accessor p = await 0 }
