class Foo {
  #tag = function() { return this };

  getReceiver() {
    return this.#tag`tagged template`;
  }
}
