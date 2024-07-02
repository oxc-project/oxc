export class Foo {
  private constructor(a: number = 0) {}
}

export class Bar {
  public constructor(a: number = 0) {}
}

export class Zoo { foo<F>(f: F): F { return f } }

export abstract class Qux {
  abstract foo(): void;
  bar(): void {}
  baz(): void {}
}