class S {}

class C extends S {
  constructor() {
    if (true) {
      const _super = super();
      this.fn = async () => { return [this, 1]; };
    }

    super();
    async () => { return [this, 2]; };
  }
}

class C2 extends S {
  constructor() {
    if (true) {
      const _super = super();
      this.fn = async () => [this, 1];
    }

    super();
    async () => [this, 2];
  }
}
