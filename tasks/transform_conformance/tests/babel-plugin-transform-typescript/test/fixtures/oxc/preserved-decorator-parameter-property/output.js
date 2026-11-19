class Base {
  set x(value) {
    record(value);
  }
}

@decorate
class Derived extends Base {
  constructor(x) {
    super();
    this.x = x;
  }
}

new Derived(7);
