let c;

class S {}

class C extends S {
  prop = 123;
  constructor() {
    super(c = super());
  }
}
