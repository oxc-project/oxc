const object = {
  async method() {
    return arguments;
  },
  async *generatorMethod() {
    return arguments;
  },
};

class Example {
  async method() {
    return arguments;
  }

  async *generatorMethod() {
    return arguments;
  }
}

function outer() {
  return async () => arguments;
}
