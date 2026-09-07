function makeClasses() {
  const Classes = [];
  for (let captured = 0; captured < 2; captured++) {
    Classes.push(class {
      *#read() {
        yield captured;
      }
      run() {
        return this.#read().next().value + 1;
      }
    });
  }
  return Classes;
}

const Classes = makeClasses();
expect(new Classes[0]().run()).toBe(1);
expect(new Classes[1]().run()).toBe(2);
