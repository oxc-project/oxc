{ function f() {} }
f();

function t() {
    { function g() {} }
    return g;
}

{
    type x = number;
    function x() {}
}
x();

try {} catch (e) { if (true) function e() {} }
if (Math.random() > 0.5) function h() {} else function h() {}
