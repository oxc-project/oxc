class Base {
  set x(value: number) {
    record(value);
  }
}

@decorate
class Derived extends Base {
  constructor(public x: number) {
    super();
  }
}

new Derived(7);
