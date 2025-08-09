// The arrow functions in this test case are to make sure that scopes are re-parented correctly
function f(a = () => {}, b = () => {}) {
  using x = a(), y = b();
  doSomethingWith(x, y, () => {});
}
