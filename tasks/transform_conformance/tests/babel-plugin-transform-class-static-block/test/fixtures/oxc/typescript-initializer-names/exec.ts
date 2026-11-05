class Names {
  static {}
  static asFunction = (function () {} as any);
  static {}
  static assertedArrow = (<any>(() => {}));
  static {}
  static satisfiesFunction = (function () {} satisfies Function);
  static {}
  static nonNullClass = class { static seen = this.name; }!;
  static {}
  static asClass = (class { static seen = this.name; } as any);
  static {}
  static satisfiesClass = (class { static seen = this.name; } satisfies { new(): unknown; seen: string });
  static {}
  static namedFunction = (function Explicit() {} as any);
  static {}
  static nestedClass = (((class { static seen = this.name; }) satisfies { new(): unknown; seen: string }) as any);
  static {}
}
expect(Names.asFunction.name).toBe("asFunction");
expect(Names.assertedArrow.name).toBe("assertedArrow");
expect(Names.satisfiesFunction.name).toBe("satisfiesFunction");
expect([Names.nonNullClass.name, Names.nonNullClass.seen]).toEqual(["nonNullClass", "nonNullClass"]);
expect([Names.asClass.name, Names.asClass.seen]).toEqual(["asClass", "asClass"]);
expect([Names.satisfiesClass.name, Names.satisfiesClass.seen]).toEqual(["satisfiesClass", "satisfiesClass"]);
expect(Names.namedFunction.name).toBe("Explicit");
expect([Names.nestedClass.name, Names.nestedClass.seen]).toEqual(["nestedClass", "nestedClass"]);

function defaultClass(C = (class {
  static value = 0;
  static { this.seen = this.name; }
} as any)) { return C; }
const Default = defaultClass();
expect([Default.name, Default.seen]).toEqual(["C", "C"]);
const objectClasses = {
  ObjectClass: (class {
    static value = 0;
    static { this.seen = this.name; }
  } as any),
};
expect([objectClasses.ObjectClass.name, objectClasses.ObjectClass.seen]).toEqual(["ObjectClass", "ObjectClass"]);
const instanceKey = "instance";
class Holder {
  [instanceKey] = (class {
    static value = 0;
    static { this.seen = this.name; }
  } as any);
}
const Instance = new Holder().instance;
expect([Instance.name, Instance.seen]).toEqual(["instance", "instance"]);
function* suspendedFields() {
  const Holder = (class {
    [yield "key"] = (class {
      static value = 0;
      static { this.seen = this.name; }
    } as any);
  } as any);
  return Holder;
}
const generator = suspendedFields();
expect(generator.next().value).toBe("key");
const Suspended = generator.next("field").value;
expect(Suspended.name).toBe("Holder");
const Field = new Suspended().field;
expect([Field.name, Field.seen]).toEqual(["field", "field"]);
