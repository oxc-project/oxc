const makeClass = () => class {
  #field = 41;
  *#read() {
    yield this.#field;
  }
  run() {
    return this.#read().next().value + 1;
  }
};

export function get() {
  return new (makeClass())().run();
}
