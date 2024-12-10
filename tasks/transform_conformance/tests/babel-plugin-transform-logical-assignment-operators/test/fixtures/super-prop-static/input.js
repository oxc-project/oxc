class C extends S {
  method() {
    super.prop &&= 1;
    super.prop ||= 2;
    super.prop ??= 3;
  }
}
