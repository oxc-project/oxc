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

export function get() {
  const Classes = makeClasses();
  return [new Classes[0]().run(), new Classes[1]().run()];
}
