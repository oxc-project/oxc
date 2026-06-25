// Expected output: only used imports are transformed
var a = foo.a;
var b = a.b;
var c = b.c;
export let bar = c;
