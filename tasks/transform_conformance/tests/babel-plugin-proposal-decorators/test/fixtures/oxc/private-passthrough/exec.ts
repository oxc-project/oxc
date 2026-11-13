const keep = () => {};

class PlainOuter {
  #value = 1;
  constructor(public outer = 10) {}

  create() {
    return class DecoratedInner {
      @keep #inner = 2;
      constructor(public inner = 20) {}
      read(outer: PlainOuter) {
        return outer.#value + this.#inner + this.inner;
      }
    };
  }
}

const plainOuter = new PlainOuter();
const FirstInner = plainOuter.create();
expect(plainOuter.outer).toBe(10);
expect(new FirstInner().read(plainOuter)).toBe(23);

@keep
class DecoratedOuter {
  #value = 3;
  constructor(public outer = 30) {}
  read() {
    return this.#value;
  }

  create() {
    return class PlainMiddle {
      #value = 4;
      constructor(public middle = 40) {}
      create() {
        return class DecoratedInner {
          @keep #inner = 5;
          constructor(public inner = 50) {}
          read(middle: PlainMiddle) {
            return middle.#value + this.#inner + middle.middle + this.inner;
          }
        };
      }
    };
  }
}

const decoratedOuter = new DecoratedOuter();
const Middle = decoratedOuter.create();
const middle = new Middle();
const SecondInner = middle.create();
expect(decoratedOuter.outer).toBe(30);
expect(decoratedOuter.read()).toBe(3);
expect(new SecondInner().read(middle)).toBe(99);
