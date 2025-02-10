class S {}

class C extends S {
  constructor() {
    super(async () => {
      this;
    });
  }
}

class C2 extends S {
  constructor() {
    super(async () => this);
  }
}
