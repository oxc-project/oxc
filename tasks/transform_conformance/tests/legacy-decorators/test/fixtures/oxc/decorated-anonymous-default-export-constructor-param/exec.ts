let decorated: any;
function dec(target: any): any {
  decorated = target;
}

export default class {
  static value = 42;
  static #secret = 7;
  static self = this;
  static fromInitializer = this.#secret;
  static readSecret() {
    return this.#secret;
  }
  static {
    (this as any).fromBlock = this.value;
  }
  field = 1;
  constructor(@dec value?: number) {}
}

expect(decorated.value).toBe(42);
expect(decorated.self).toBe(decorated);
expect(decorated.fromInitializer).toBe(7);
expect(decorated.readSecret()).toBe(7);
expect(decorated.fromBlock).toBe(42);
expect(new decorated().field).toBe(1);
