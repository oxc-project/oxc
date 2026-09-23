class C {
  readonly field = 1;
  constructor(value: number);
  constructor(public readonly value: number) {}
}

class D {
  protected constructor() {}
  readonly = 1;
}

declare class E {
  readonly field: number;
  constructor();
}
