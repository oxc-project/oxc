const outer = {
  value: 0,
  async method() {
    () => super.value;

    const inner = {
      value: 0,
      async method() {
        () => super.value;
      }
    };

    () => super.value;
  }
};
