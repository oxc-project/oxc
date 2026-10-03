var _staticBlock;
class A {
  static x = (_staticBlock = () => this.y = 1, Object.preventExtensions(this));
}
_staticBlock();
