const A = class { static { this.seen = this.name; } };
expect([A.name, A.seen]).toEqual(["A", "A"]);
let assigned;
assigned = class { static { this.seen = this.name; } };
expect([assigned.name, assigned.seen]).toEqual(["assigned", "assigned"]);
let logical;
logical ||= class { static { this.seen = this.name; } };
expect([logical.name, logical.seen]).toEqual(["logical", "logical"]);
function defaults(Param = class { static { this.seen = this.name; } }) {
  return [Param.name, Param.seen];
}
expect(defaults()).toEqual(["Param", "Param"]);
const { value: Destructured = class { static { this.seen = this.name; } } } = {};
expect([Destructured.name, Destructured.seen]).toEqual(["Destructured", "Destructured"]);
let target;
({ target = class { static { this.seen = this.name; } } } = {});
expect([target.name, target.seen]).toEqual(["target", "target"]);
let other;
[other = class { static { this.seen = this.name; } }] = [];
expect([other.name, other.seen]).toEqual(["other", "other"]);
const named = class Explicit { static { this.seen = this.name; } };
expect([named.name, named.seen]).toEqual(["Explicit", "Explicit"]);
const anonymous = (0, class { static { this.seen = this.name; } });
expect([anonymous.name, anonymous.seen]).toEqual(["", ""]);
const object = {
  entry: class { static { this.seen = this.name; } },
  [Symbol.for("key")]: class { static { this.seen = this.name; } },
  ["__proto__"]: class { static { this.seen = this.name; } },
};
expect([object.entry.name, object.entry.seen]).toEqual(["entry", "entry"]);
expect([object[Symbol.for("key")].name, object[Symbol.for("key")].seen]).toEqual(["[key]", "[key]"]);
expect(object.__proto__.seen).toBe("__proto__");
const proto = { __proto__: class { static { this.seen = this.name; } } };
expect(Object.getPrototypeOf(proto).seen).toBe("");
class Fields {
  Child = class { static { this.seen = this.name; } };
  static Child = class { static { this.seen = this.name; } };
}
expect(new Fields().Child.seen).toBe("Child");
expect(Fields.Child.seen).toBe("Child");
const receiver = {};
receiver.member = class { static { this.seen = this.name; } };
expect([receiver.member.name, receiver.member.seen]).toEqual(["", ""]);
[receiver.defaulted = class { static { this.seen = this.name; } }] = [];
expect([receiver.defaulted.name, receiver.defaulted.seen]).toEqual(["", ""]);
