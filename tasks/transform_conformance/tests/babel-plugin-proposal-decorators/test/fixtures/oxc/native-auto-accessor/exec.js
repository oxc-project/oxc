class Plain {
  accessor value = 1;
  #other = 2;
  field = 3;
  static field = 4;
  read() { return this.#other; }
}
const plain = new Plain();
plain.value = 5;
expect([plain.value, plain.read(), plain.field, Plain.field]).toEqual([5, 2, 3, 4]);

class Static {
  static accessor value = 1;
}
class Derived extends Static {}
Derived.value = 2;
expect(Static.value).toBe(2);
const original = Static;
Static = null;
const descriptor = Object.getOwnPropertyDescriptor(original, "value");
expect(descriptor.get.call({})).toBe(2);
descriptor.set.call({}, 3);
expect(original.value).toBe(3);

const Named = class {
  static accessor value = this.name;
  static observedName = this.name;
};
expect([Named.name, Named.value, Named.observedName]).toEqual(["Named", "Named", "Named"]);

const classes = [];
for (let i = 0; i < 2; i++) {
  classes.push(class { static accessor value = i; });
}
classes[0].value = 3;
expect([classes[0].name, classes[0].value, classes[1].value]).toEqual(["", 3, 1]);

const key = Symbol("named");
const object = {
  [key]: class { static accessor value = this.name; },
};
expect(object[key].value).toBe("[named]");

function withDefault(C = class { static accessor value = this.name; }) { return C; }
expect(withDefault().value).toBe("C");

class Holder {
  accessor item = class { static accessor value = this.name; };
  accessor plain = class { static value = this.name; };
  accessor callback = function () {};
  accessor arrow = () => {};
  field = class { static accessor value = this.name; };
}
const holder = new Holder();
expect([holder.item.name, holder.item.value, holder.field.value]).toEqual(["item", "item", "field"]);
expect([holder.plain.name, holder.plain.value, holder.callback.name, holder.arrow.name]).toEqual(["plain", "plain", "callback", "arrow"]);

let coercions = 0;
const computedKey = { [Symbol.toPrimitive]() { coercions++; return "value"; } };
class Computed {
  accessor [computedKey] = 1;
}
expect(coercions).toBe(1);
const computed = new Computed();
computed.value = 2;
expect(computed.value).toBe(2);

class Private {
  #_value_accessor_storage = 1;
  accessor #value = 2;
  accessor value = 3;
  read() { return [this.#_value_accessor_storage, this.#value, this.value]; }
}
expect(new Private().read()).toEqual([1, 2, 3]);

const computedClasses = [];
for (let i = 0; i < 2; i++)
  computedClasses.push(class { accessor ["item" + i] = function () {}; });
expect([new computedClasses[0]().item0.name, new computedClasses[1]().item1.name]).toEqual(["item0", "item1"]);

const nestedClasses = [];
for (let i = 0; i < 2; i++) {
  nestedClasses.push(class {
    accessor ["item" + i] = class { static accessor nameAtInitialization = this.name; };
  });
}
expect([new nestedClasses[0]().item0.nameAtInitialization, new nestedClasses[1]().item1.nameAtInitialization]).toEqual(["item0", "item1"]);

function computedDefault(key, C = class { accessor [key] = function () {}; }) { return C; }
const FirstDefault = computedDefault("first");
const SecondDefault = computedDefault("second");
expect([FirstDefault.name, new FirstDefault().first.name, SecondDefault.name, new SecondDefault().second.name]).toEqual(["C", "first", "C", "second"]);
