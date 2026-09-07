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

expect(new (makeClass(41))().run()).toBe(42);
