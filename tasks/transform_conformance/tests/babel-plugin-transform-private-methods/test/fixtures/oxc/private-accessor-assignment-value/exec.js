class Counter {
  #value = 1;
  writes = [];

  get #count() {
    return this.#value;
  }

  set #count(value) {
    this.writes.push(value);
    this.#value = value === 100 ? 0 : value;
    return "from setter";
  }

  static #total = 1;

  static get #sum() {
    return this.#total;
  }

  static set #sum(value) {
    this.#total = value;
  }

  add(k) {
    return (this.#count += k);
  }

  increment() {
    return ++this.#count;
  }

  decrement() {
    return --this.#count;
  }

  andAssign(value) {
    return (this.#count &&= value);
  }

  orAssign(value) {
    return (this.#count ||= value);
  }

  nullishAssign(value) {
    return (this.#count ??= value);
  }

  addTo(getObject, k) {
    return (getObject().#count += k);
  }

  inArguments(k) {
    return [(this.#count += k), ++this.#count];
  }

  statement(k) {
    this.#count += k;
    ++this.#count;
  }

  static add(k) {
    return (this.#sum += k);
  }

  static increment() {
    return ++this.#sum;
  }

  static addTo(getObject, k) {
    return (getObject().#sum += k);
  }

  static addToClass(k) {
    return (Counter.#sum += k);
  }
}

const counter = new Counter();
expect(counter.add(2)).toBe(3);
expect(counter.increment()).toBe(4);
expect(counter.decrement()).toBe(3);
expect(counter.andAssign(5)).toBe(5);
expect(counter.orAssign(9)).toBe(5);
expect(counter.nullishAssign(9)).toBe(5);

// The value is the right side, not the value that the setter stores or returns.
expect(counter.add(95)).toBe(100);
expect(counter.orAssign(7)).toBe(7);

expect(counter.andAssign(0)).toBe(0);
expect(counter.orAssign(4)).toBe(4);
expect(counter.andAssign(null)).toBe(null);
expect(counter.nullishAssign(1)).toBe(1);
expect(counter.inArguments(1)).toEqual([2, 3]);

let calls = 0;
const getCounter = () => {
  calls++;
  return counter;
};
expect(counter.addTo(getCounter, 1)).toBe(4);
expect(calls).toBe(1);

const writes = counter.writes.length;
counter.statement(1);
expect(counter.writes.length).toBe(writes + 2);
expect(counter.add(0)).toBe(6);

expect(() => counter.addTo(() => ({}), 1)).toThrow(TypeError);

expect(Counter.add(2)).toBe(3);
expect(Counter.increment()).toBe(4);
expect(Counter.addTo(() => Counter, 1)).toBe(5);
expect(Counter.addToClass(1)).toBe(6);

// A nested call must not change the value of the outer assignment.
class Recursive {
  value = 0;

  get #count() {
    return this.value;
  }

  set #count(value) {
    this.value = value;
    if (value === 1) this.run();
  }

  run(result = (this.#count += 1)) {
    return result;
  }
}
expect(new Recursive().run()).toBe(1);

// The expression body of an arrow function is the return value.
class Arrow {
  #value = 1;

  get #count() {
    return this.#value;
  }

  set #count(value) {
    this.#value = value;
  }

  add = (k) => (this.#count += k);
  increment = () => ++this.#count;
  orAssign = (value) => (this.#count ||= value);

  adder() {
    return (k) => (this.#count += k);
  }

  incrementer() {
    return () => ++this.#count;
  }

  orAssigner() {
    return (value) => (this.#count ||= value);
  }
}
const arrow = new Arrow();
expect(arrow.add(1)).toBe(2);
expect(arrow.increment()).toBe(3);
expect(arrow.orAssign(9)).toBe(3);
expect(arrow.adder()(1)).toBe(4);
expect(arrow.incrementer()()).toBe(5);
expect(arrow.orAssigner()(9)).toBe(5);
