const keep = () => {};

class PlainOuter {
  #map = new Map([["key", { value: 7 }]]);

  create() {
    const outer = this;
    @keep
    class DecoratedInner {
      #token = 1;

      read() {
        return outer.#map?.get("key")?.value + this.#token;
      }

      readOptional(object = outer) {
        return object?.#map.get("key").value;
      }

      remove() {
        return delete outer.#map?.get("key").value;
      }
    }
    return new DecoratedInner();
  }
}

const inner = new PlainOuter().create();
expect(inner.read()).toBe(8);
expect(inner.readOptional()).toBe(7);
expect(inner.readOptional(null)).toBeUndefined();
expect(inner.remove()).toBe(true);
expect(inner.readOptional()).toBeUndefined();
