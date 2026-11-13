const keep = () => {};

@keep
class Outer {
  #inner;

  create() {
    const outer = this;
    class Inner {
      #callback;
      #value = 7;
      #read() { return this.#value; }

      read(object = outer) {
        return (object?.#inner.#read)?.();
      }

      callUndefined(object = outer) {
        return (object?.#inner.#callback)?.();
      }
    }
    this.#inner = new Inner();
    return this.#inner;
  }
}

const inner = new Outer().create();
expect(inner.read()).toBe(7);
expect(inner.read(null)).toBeUndefined();
expect(inner.callUndefined()).toBeUndefined();
expect(inner.callUndefined(null)).toBeUndefined();
