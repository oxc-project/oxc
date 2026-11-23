class Counter {
  #value = 1;

  get #count() {
    return this.#value;
  }

  set #count(value) {
    this.#value = value;
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

  andAssign(value) {
    return (this.#count &&= value);
  }

  nullishAssign(getObject, value) {
    return (getObject().#count ??= value);
  }

  statement(k) {
    this.#count += k;
    ++this.#count;
    this.#count ||= k;
  }

  static add(k) {
    return (this.#sum += k);
  }

  static addToClass(k) {
    return (Counter.#sum += k);
  }
}
