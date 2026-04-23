let _Symbol$toPrimitive;
export class SampleClass {
  static {
    _Symbol$toPrimitive = Symbol.toPrimitive;
  }
  static {
    this[_Symbol$toPrimitive] = "test";
  }
  constructor() {}
}
