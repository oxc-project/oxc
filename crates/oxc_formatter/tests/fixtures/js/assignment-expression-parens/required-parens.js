new (a = b)();
new (a = b)(arg);

(a = b).prop;
(a = b)["key"];

async function f() { await (a = b); }

!(a = b);
-(a = b);
typeof (a = b);

(a = b) + c;
c * (a = b);

(a = b) && c;
c || (a = b);

(a = b) ? c : d;

class C { [(a = 1)] = 0; }

foo((a = b));
new Foo((a = b));

function g() { return (a = b); }
function h() { throw (a = b); }
function* k() { yield (a = b); }

c ? (a = b) : d;
c ? d : (a = b);
