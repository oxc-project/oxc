const keep = () => {};

@keep
class DecoratedOuter {
  #inner;

  create() {
    const outer = this;
    class PlainInner {
      #outer = outer;
      #callback;
      #value = { value: 7, read() { return this.value; } };
      #read() { return this.#value.value; }

      read() {
        return this.#value?.read?.();
      }

      readOptional(object = this) {
        return object?.#value.value;
      }

      readLoweredFirst(object = outer) {
        return object?.#inner.#value;
      }

      readNativeFirst(object = this) {
        return object?.#outer.#inner;
      }

      callNative(object = outer) {
        return object?.#inner.#read();
      }

      callParenthesized(object = outer) {
        return (object?.#inner.#read)();
      }

      callParenthesizedOptional(object = outer) {
        return (object?.#inner.#read)?.();
      }

      callUndefined(object = outer) {
        return (object?.#inner.#callback)?.();
      }

      remove() {
        return delete this.#value?.value;
      }
    }
    this.#inner = new PlainInner();
    return this.#inner;
  }
}

const inner = new DecoratedOuter().create();
expect(inner.read()).toBe(7);
expect(inner.readOptional()).toBe(7);
expect(inner.readOptional(null)).toBeUndefined();
expect(inner.readLoweredFirst().value).toBe(7);
expect(inner.readLoweredFirst(null)).toBeUndefined();
expect(inner.readNativeFirst()).toBe(inner);
expect(inner.readNativeFirst(null)).toBeUndefined();
expect(inner.callNative()).toBe(7);
expect(inner.callNative(null)).toBeUndefined();
expect(inner.callParenthesized()).toBe(7);
expect(inner.callParenthesizedOptional()).toBe(7);
expect(inner.callParenthesizedOptional(null)).toBeUndefined();
expect(inner.callUndefined()).toBeUndefined();
expect(inner.callUndefined(null)).toBeUndefined();
expect(inner.remove()).toBe(true);
expect(inner.readOptional()).toBeUndefined();
