let receiver;

class Accessor {
  get #value() {
    receiver = {};
    return 1;
  }

  set #value(value) {}

  run(rhs) {
    return (receiver.#value &&= rhs());
  }

  parameter(result = [receiver.#value &&= rhs()]) {
    return result;
  }
}
