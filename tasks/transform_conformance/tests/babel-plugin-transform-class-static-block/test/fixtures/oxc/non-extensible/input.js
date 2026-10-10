class A {
  static x = Object.preventExtensions(this);
  static {
    this.y = 1;
  }
}
