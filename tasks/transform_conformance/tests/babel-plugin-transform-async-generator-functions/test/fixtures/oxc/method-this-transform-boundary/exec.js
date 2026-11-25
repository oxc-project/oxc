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

return component.method().then(function (value) {
  expect(value).toBe("undefined");
  return component.generatorMethod().next();
}).then(function (result) {
  expect(result.value).toBe(42);
});
