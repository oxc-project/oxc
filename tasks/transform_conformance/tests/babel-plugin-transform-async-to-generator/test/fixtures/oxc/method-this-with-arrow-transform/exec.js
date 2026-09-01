const component = {
  value: 42,
  async method() {
    return this.value;
  },
};

class Component {
  constructor() {
    this.value = 42;
  }
  async method() {
    return this.value;
  }
}

return component.method().then(function (value) {
  expect(value).toBe(42);
  return new Component().method();
}).then(function (value) {
  expect(value).toBe(42);
});
