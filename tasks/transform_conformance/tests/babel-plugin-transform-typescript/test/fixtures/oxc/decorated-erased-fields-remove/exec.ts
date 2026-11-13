const keep = () => {};
let keyRuns = 0;
let extraRuns = 0;
function initialize(value, context) {
  context.addInitializer(() => extraRuns++);
  return () => 9;
}

class Base {
  static value = 6;
  constructor() { this.x = 5; }
}

class Plain extends Base {
  x: number;
  static value: number;
}

class Decorated extends Base {
  x: number;
  static value: number;
  #brand;
  ["erased"]: number;
  [(() => { keyRuns++; return "computed"; })()]: number;
  @initialize y: number;
  @keep method() {}
  hasBrand() { return #brand in this; }
}

const plain = new Plain();
const decorated = new Decorated();
expect(plain.x).toBe(5);
expect(decorated.x).toBe(plain.x);
expect(Decorated.value).toBe(Plain.value);
expect(Object.hasOwn(decorated, "erased")).toBe(false);
expect(Object.hasOwn(decorated, "computed")).toBe(true);
expect(keyRuns).toBe(1);
expect(decorated.hasBrand()).toBe(true);
expect(decorated.y).toBe(9);
expect(extraRuns).toBe(1);
