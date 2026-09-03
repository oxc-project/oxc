const makeClass = () => class {
  #field = 41;
  #read() {
    return this.#field;
  }
  run() {
    return this.#read() + 1;
  }
};

expect(new (makeClass())().run()).toBe(42);
