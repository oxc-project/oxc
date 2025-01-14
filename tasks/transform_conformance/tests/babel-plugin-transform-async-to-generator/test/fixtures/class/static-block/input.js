class C extends S {
  static {
    this.fn = async () => super.foo;
  }
}
