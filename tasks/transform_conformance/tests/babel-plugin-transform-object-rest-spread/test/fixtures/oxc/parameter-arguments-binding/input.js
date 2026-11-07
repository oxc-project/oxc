function f({ a, ...rest }, b = a, c = arguments[0]) {
  var arguments;
  return [arguments[0].a, c.a, b];
}
function* generator({ a, ...rest }, b = a) { var arguments; yield [arguments[0].a, b]; }
function* bound({ a, ...rest }, arguments = a) { yield arguments; }
