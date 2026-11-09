function f({ a, ...rest }, b = a, c = arguments[0]) {
  var arguments;
  return [arguments[0].a, c.a, b];
}
function lexicalLet({ a, ...rest }, b = a) {
  let arguments = 1;
  return arguments;
}
function lexicalConst({ a, ...rest }, b = a, c = arguments[0]) {
  const arguments = 1;
  return [arguments, b, c.a];
}
function lexicalDestructured({ a, ...rest }, b = a, c = arguments[0]) {
  let { value: arguments } = { value: 1 };
  return [arguments, b, c.a];
}
function bodyFunction({ a, ...rest }, b = a, c = arguments[0]) {
  function arguments() { return 1; }
  return [arguments(), b, c.a];
}
