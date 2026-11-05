const component = {
  value: 42,
  async method() {
    this.value;
    return eval("typeof _this");
  },
  async *generatorMethod() {
    yield this.value;
  },
};
