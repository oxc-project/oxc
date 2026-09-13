const makeClass = () => class {
  #field = 41;
  *#read() {
    yield this.#field;
  }
  run() {
    return this.#read().next().value + 1;
  }
};

expect(new (makeClass())().run()).toBe(42);
