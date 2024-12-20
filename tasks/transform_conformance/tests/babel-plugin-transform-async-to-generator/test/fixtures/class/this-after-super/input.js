class C extends S {
  constructor() {
    if (condition) {
      const _super = super()
      this.fn = async () => { return [this, 1]; };
    }

    super()
    async () => { return [this, 2]; };
  }
}
