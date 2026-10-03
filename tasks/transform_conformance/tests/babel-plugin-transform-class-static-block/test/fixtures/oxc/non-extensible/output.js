class A {
  static x = (Object.preventExtensions(this), void (this.y = 1));
}
