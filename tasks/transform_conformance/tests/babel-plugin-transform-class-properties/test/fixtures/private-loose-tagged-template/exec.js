class Foo {
  #tag = function() { return this };

  getReceiver() {
    return this.#tag`tagged template`;
  }
}

const foo = new Foo();
const receiver = foo.getReceiver();
expect(receiver).toBe(foo);
