const keep = () => {};

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
  @keep method() {}
}

expect(new Plain().x).toBe(5);
expect(new Decorated().x).toBe(5);
expect(Plain.value).toBe(6);
expect(Decorated.value).toBe(6);
