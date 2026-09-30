let g = 0;

function f(a, b = a, c = g) {
  var g = 1;
  return [a, b, c];
}
