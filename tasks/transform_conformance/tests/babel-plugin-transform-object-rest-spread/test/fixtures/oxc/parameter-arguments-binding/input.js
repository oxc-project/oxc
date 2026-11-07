function f({ a, ...rest }, b = a, c = arguments[0]) {
  var arguments;
  return [arguments[0].a, c.a, b];
}
