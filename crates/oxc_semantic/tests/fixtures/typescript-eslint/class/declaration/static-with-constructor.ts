// https://github.com/typescript-eslint/typescript-eslint/issues/5577
function f() {}

class A {
  static {}

  constructor() {
    f();
  }
}
