let f;

class S {}

class C extends S {
  constructor(x) {
    super(super(), this.x = x, f = async () => this);
  }
}
