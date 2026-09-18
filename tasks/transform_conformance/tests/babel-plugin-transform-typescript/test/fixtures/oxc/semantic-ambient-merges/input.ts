declare function f(x: string): void;
function f(x: string) {}
function g(x: string) {}
declare function g(x: string): void;
declare var v: number;
var v = 1;
var w = 2;
declare var w: number;
f("a"); g("b"); use(v, w);
