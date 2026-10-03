const classes = [];
for (const key of ['a', 'b']) {
  classes.push(class { [key] = class { static { this.seen = this.name; } }; });
}
expect(new classes[0]().a.seen).toBe("a");
expect(new classes[1]().b.seen).toBe("b");
let nextKey = "first";
class Factory {
  Outer = class { [nextKey] = class { static { this.seen = this.name; } }; };
}
const First = new Factory().Outer;
nextKey = "second";
const Second = new Factory().Outer;
expect(new First().first.seen).toBe("first");
expect(new Second().second.seen).toBe("second");
function parameter(Outer = class { [nextKey] = class { static { this.seen = this.name; } }; }) {
  return Outer;
}
nextKey = "third";
const Third = parameter();
nextKey = "fourth";
const Fourth = parameter();
expect(new Third().third.seen).toBe("third");
expect(new Fourth().fourth.seen).toBe("fourth");
const repeated = [];
while (repeated.length < 2 && repeated.push(class {
  [String(repeated.length)] = class { static { this.seen = this.name; } };
})) {}
expect(new repeated[0]()["0"].seen).toBe("0");
expect(new repeated[1]()["1"].seen).toBe("1");
function* suspended() {
  const result = [];
  for (const key of ["a", "b"]) {
    result.push(class { [yield key] = class { static { this.seen = this.name; } }; });
  }
  return result;
}
const generator = suspended();
expect(generator.next().value).toBe("a");
expect(generator.next("a").value).toBe("b");
const suspendedClasses = generator.next("b").value;
expect(new suspendedClasses[0]().a.seen).toBe("a");
expect(new suspendedClasses[1]().b.seen).toBe("b");

function* suspendedHeader() {
  const result = [];
  while (result.length < 2 && result.push(class {
    [yield result.length] = class { static { this.seen = this.name; } };
    static { Object.freeze(this); }
  })) {}
  return result;
}
const headerGenerator = suspendedHeader();
expect(headerGenerator.next().value).toBe(0);
expect(headerGenerator.next("a").value).toBe(1);
const headerClasses = headerGenerator.next("b").value;
expect(new headerClasses[0]().a.seen).toBe("a");
expect(new headerClasses[1]().b.seen).toBe("b");
expect(headerClasses[0].name).toBe("");
expect(Object.isFrozen(headerClasses[0])).toBe(true);
function* suspendedName() {
  const Named = class {
    [yield "key"] = class { static {} };
    static seen = this.name;
  };
  const Method = class {
    [yield "key"] = class { static {} };
    static ["na" + "me"]() {}
    static seen = this.name;
  };
  const Getter = class {
    [yield "key"] = class { static {} };
    static get ["na" + "me"]() { return "getter"; }
    static seen = this.name;
  };
  class Declaration {
    [yield "key"] = class { static {} };
  }
  const Original = Declaration;
  Declaration = null;
  return [Named, Method, Getter, Original];
}
const namesGenerator = suspendedName();
namesGenerator.next();
namesGenerator.next("a");
namesGenerator.next("a");
namesGenerator.next("a");
const [Named, Method, Getter, Original] = namesGenerator.next("a").value;
expect(Named.seen).toBe("Named");
expect(Method.seen).toBe(Method.name);
expect(typeof Method.seen).toBe("function");
expect(Getter.seen).toBe("getter");
expect(new Original().a.name).toBe("a");
