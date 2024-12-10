class C extends S {
  method() {
    super.prop && (super.prop = 1);
    super.prop || (super.prop = 2);
    super.prop ?? (super.prop = 3);
  }
}
