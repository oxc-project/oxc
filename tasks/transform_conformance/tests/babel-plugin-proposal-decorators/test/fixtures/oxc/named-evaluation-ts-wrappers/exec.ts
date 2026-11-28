// TypeScript wrappers are erased, so they keep named evaluation of decorated classes.
const dec = (value: any) => value;
const object = {
  asAny: @dec class {} as any,
  nonNull: (@dec class {})!,
  satisfies: @dec class {} satisfies object,
  nested: ((@dec class {}) as unknown) as any,
};
expect(object.asAny.name).toBe("asAny");
expect(object.nonNull.name).toBe("nonNull");
expect(object.satisfies.name).toBe("satisfies");
expect(object.nested.name).toBe("nested");

const declared = @dec class {} as any;
expect(declared.name).toBe("declared");

class Holder {
  field = @dec class {} as any;
}
expect(new Holder().field.name).toBe("field");
