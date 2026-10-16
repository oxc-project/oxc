const dec = (_: unknown, ctx: ClassFieldDecoratorContext) => (v: number) => v + 1;

abstract class Base {
  abstract kind: string;
  abstract method(): void;
}

class Foo extends Base {
  declare readonly meta: string;
  kind = "foo";
  @dec count: number = 1;
  label!: string;
  optional?: number;
  method(): void;
  method(x?: number): void {}
}

const foo = new Foo();
expect(foo.count).toBe(2);
expect("meta" in foo).toBe(false);
expect(Object.keys(foo)).toEqual(["kind", "count", "label", "optional"]);
