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

function defaultClass(C = (class {
  static value = 0;
  static { this.seen = this.name; }
} as any)) { return C; }
const Default = defaultClass();
const objectClasses = {
  ObjectClass: (class {
    static value = 0;
    static { this.seen = this.name; }
  } as any),
};
const instanceKey = "instance";
class Holder {
  [instanceKey] = (class {
    static value = 0;
    static { this.seen = this.name; }
  } as any);
}
function* suspendedFields() {
  const Holder = (class {
    [yield "key"] = (class {
      static value = 0;
      static { this.seen = this.name; }
    } as any);
  } as any);
  return Holder;
}
