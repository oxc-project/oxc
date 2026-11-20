class Counter {
  static #count = 10;

  static add(k) {
    return (this.#count += k);
  }

  static subtract(getObject, k) {
    getObject().#count -= k;
    return this.#count;
  }

  static addTo(obj) {
    obj.#count += 1;
  }
}

let calls = 0;
const getCounter = () => {
  calls++;
  return Counter;
};

expect(Counter.add(1)).toBe(11);
expect(Counter.subtract(getCounter, 2)).toBe(9);
expect(calls).toBe(1);
expect(() => Counter.addTo({})).toThrow(TypeError);
expect(Counter.add(0)).toBe(9);
