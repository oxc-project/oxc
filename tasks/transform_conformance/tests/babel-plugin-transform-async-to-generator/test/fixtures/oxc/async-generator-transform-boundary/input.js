class Base {
  get value() {
    return 1;
  }
}

class Derived extends Base {
  async *values() {
    super.value;
    yield eval("typeof _superprop_getValue");
  }
}
