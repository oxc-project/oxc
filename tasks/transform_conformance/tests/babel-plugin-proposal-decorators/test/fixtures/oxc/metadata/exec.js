const META = Symbol.metadata ?? Symbol.for("Symbol.metadata");
const dec = (value, ctx) => {
  ctx.metadata[ctx.name] = ctx.kind;
};

@dec
class Base {
  @dec method() {}
  @dec field;
}

class Derived extends Base {
  @dec accessor acc;
}

class Undecorated extends Base {}

expect(Base[META].method).toBe("method");
expect(Base[META].field).toBe("field");
expect(Base[META].Base).toBe("class");
expect(Derived[META].acc).toBe("accessor");
expect(Derived[META].method).toBe("method");
expect(Object.getPrototypeOf(Derived[META])).toBe(Base[META]);
expect(Object.hasOwn(Derived[META], "method")).toBe(false);
expect(Undecorated[META]).toBe(Base[META]);
