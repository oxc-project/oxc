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
