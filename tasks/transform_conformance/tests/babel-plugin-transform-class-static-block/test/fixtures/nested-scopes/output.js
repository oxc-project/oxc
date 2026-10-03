var _staticBlock;
let x, y;
class C {
  static #_ = _staticBlock = () => (x = (() => this)(), (() => {
    if (true) {
      y = this;
      z = this;
    }
  })());
}
_staticBlock();
