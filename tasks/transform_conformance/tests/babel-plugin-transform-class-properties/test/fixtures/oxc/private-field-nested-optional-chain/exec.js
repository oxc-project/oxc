class Cache {
  #items = new Map();
  #callback = function () { return this; };
  constructor(items) {
    this.#items.set("key", items);
  }
  runNext(key) {
    return (this.#items.get(key)?.find(x => x.paused))?.continue()
      ?? Promise.resolve();
  }
  getCallbackReceiver() {
    return (this?.#callback)();
  }
  callNext(key) {
    return (this.#items.get(key)?.find(x => x.paused))?.();
  }
}

const item = {
  paused: true,
  continue() { return this; },
};
const cache = new Cache([item]);
expect(cache.runNext("missing")).toBeInstanceOf(Promise);
expect(cache.runNext("key")).toBe(item);
expect(new Cache([]).runNext("key")).toBeInstanceOf(Promise);
expect(cache.getCallbackReceiver()).toBe(cache);

let calls = 0;
const callback = function () {
  calls++;
  return this;
};
callback.paused = true;
const callbacks = new Cache([callback]);
expect(callbacks.callNext("missing")).toBeUndefined();
expect(callbacks.callNext("key")).toBeUndefined();
expect(calls).toBe(1);
