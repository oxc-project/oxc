const makeClass = () => class {
  #field = 41;
  #read() {
    return this.#field;
  }
  run() {
    return this.#read() + 1;
  }
};

export function get() {
  return new (makeClass())().run();
}
