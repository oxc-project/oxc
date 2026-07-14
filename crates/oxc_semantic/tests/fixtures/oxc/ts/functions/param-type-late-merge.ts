interface X {
  a: number;
}
function g() {
  let X = 1;
  function f(a: X) {}
  interface X {
    b: number;
  }
}
