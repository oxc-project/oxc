function makeClass(value) {
  {
    const captured = value;
    return class {
      *#read() {
        yield captured;
      }
      run() {
        return this.#read().next().value + 1;
      }
    };
  }
}

export function get() {
  return new (makeClass(41))().run();
}
