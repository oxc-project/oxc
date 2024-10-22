let x, y;

class C {
  static #_ = x = (() => this)();
  static #_2 = (() => {
    if (true) {
      y = this;
      z = this;
    }
  })();
}
