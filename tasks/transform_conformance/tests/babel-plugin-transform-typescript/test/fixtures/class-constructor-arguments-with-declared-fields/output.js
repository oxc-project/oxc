export class WithStaticSameName {
  x;
  static x = 0;
  constructor(x = 1) {
    this.x = x;
  }
}
export class WithPrivateSameName {
  x;
  #x = 0;
  constructor(x = 1) {
    this.x = x;
  }
  read() {
    return this.#x;
  }
}
