class Outer extends OuterSuper {
  constructor() {
    class Inner extends super() {
      [super()] = 1;
      static [super()] = 2;

      [super()]() {}
      static [super()]() {}
    }
    let fn = async () => this;
  }
}
