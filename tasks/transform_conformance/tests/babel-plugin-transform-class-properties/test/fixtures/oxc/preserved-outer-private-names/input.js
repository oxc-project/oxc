@decorate
class Outer {
  #value = 1;
  #method() {}
  makeInner() {
    return class Inner {
      #own = 2;
      read(outer) { return [outer.#value, outer.#method(), outer?.#value, outer.#method?.(), #value in outer]; }
      write(outer, values) {
        outer.#value = 3;
        outer.#value += 1;
        outer.#value++;
        [outer.#value] = values;
        return this.#own;
      }
    };
  }
}

class AccessorOuter {
  accessor value = 1;
  #value = 2;
  makeInner() {
    return class Inner {
      #own = 3;
      read(outer) { return outer.#value + this.#own; }
    };
  }
}
