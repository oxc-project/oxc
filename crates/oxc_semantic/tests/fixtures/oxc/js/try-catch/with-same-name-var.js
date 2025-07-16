var a = 1;
try {
  throw 2
} catch (a) {
  var a = 3;
}
console.log(a);
