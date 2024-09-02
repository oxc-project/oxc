export class Foo {
  private constructor(a: number = 0) {}
}

export class Bar {
  public constructor(a: number = 0) {}
}

export class Zoo {
  foo<F>(f: F): F {
    return f;
  }
}

export abstract class Qux {
  abstract foo(): void;
  protected foo2?(): void;
  bar(): void {}
  baz(): void {}
}

export class Baz {
  readonly prop1 = 'some string';
  prop2 = 'another string';
  private prop3 = 'yet another string';
  private prop4(): void {}
  private static prop5 = 'yet another string';
  private static prop6(): void {}
}

export class Boo {
  constructor(
    public readonly prop: number = 0,
    private readonly prop2: number = 1,
    readonly prop3: number = 1,
  ) {}
}

export class Bux {
  private constructor(
    public readonly prop: number = 0,
    private readonly prop2: number = 1,
    readonly prop3: number = 1,
  ) {}
}
