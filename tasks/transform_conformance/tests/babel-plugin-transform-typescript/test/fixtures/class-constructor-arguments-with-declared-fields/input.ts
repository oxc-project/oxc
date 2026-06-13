// Static and private fields with the same name as a constructor parameter
// property are different bindings, so the parameter property still emits its
// own field declaration. (Same-name plain instance fields would be a
// `Duplicate identifier` error in TypeScript, so dedup against those is
// purely defensive.)
export class WithStaticSameName {
  static x = 0;
  constructor(public x = 1) {}
}

export class WithPrivateSameName {
  #x = 0;
  constructor(public x = 1) {}

  read() {
    return this.#x;
  }
}
