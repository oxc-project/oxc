let first, second, current, entered = false;
const keep = () => {};
function receiver() { return current; }

@keep
class C {
  #value = 0;
  get #x() {
    if (this === first && !entered) {
      entered = true;
      current = second;
      this.test();
    }
    return this.#value;
  }
  set #x(value) { this.#value = value; }
  test(value = receiver().#x ||= 1) {}
  read() { return this.#value; }
}

first = new C();
second = new C();
current = first;
first.test();
expect(first.read()).toBe(1);
expect(second.read()).toBe(1);
