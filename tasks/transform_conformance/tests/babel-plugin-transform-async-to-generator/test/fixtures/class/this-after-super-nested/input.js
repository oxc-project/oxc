class Outer {
  constructor() {
    async () => { return [this, 2]; };

    class Inner extends Outer{
      constructor() {
        if (condition) {
          const _super = super()
          this.fn = async () => { return [this, 1]; };
        }

        super()
        async () => { return [this, 2]; };
      }
    }
  }
}
