function tracked<This, V>(
  target: ClassAccessorDecoratorTarget<This, V>,
  ctx: ClassAccessorDecoratorContext<This, V>,
): ClassAccessorDecoratorResult<This, V> {
  return {
    set(v) {
      target.set.call(this, v);
      (this as any).changes = ((this as any).changes ?? 0) + 1;
    },
  };
}

class Model {
  @tracked accessor name: string = "a";
  @tracked static accessor version: number = 1;
}

const m = new Model();
m.name = "b";
m.name = "c";
expect(m.name).toBe("c");
expect((m as any).changes).toBe(2);
expect(Model.version).toBe(1);
