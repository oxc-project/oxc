class Outer {
  constructor() {
    async () => { return [this, 1]; };

    class Inner extends Outer {
      constructor() {
        if (condition) {
          const _super = super();
          this.fn = async () => { return [this, 2]; };
        }

        super();
        async () => { return [this, 3]; };
      }
    }
  }
}

class Outer2 {
  constructor() {
    async () => [this, 4];

    class Inner extends Outer2 {
      constructor() {
        if (condition) {
          const _super = super();
          this.fn = async () => [this, 5];
        }

        super();
        async () => [this, 6];
      }
    }
  }
}
