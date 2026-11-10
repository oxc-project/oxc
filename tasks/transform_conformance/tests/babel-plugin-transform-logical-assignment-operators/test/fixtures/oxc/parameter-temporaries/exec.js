class Base {
  getObject() {
    return this.object;
  }
}
class Derived extends Base {
  constructor() {
    super();
    this.object = {};
    this.value = 40;
  }
  method(key, value = (super.getObject()[key] ||= this.value + arguments.length)) {
    return value;
  }
}
const instance = new Derived();
expect(instance.method("key")).toBe(41);
expect(instance.object.key).toBe(41);
expect(instance.method("key")).toBe(41);

function getKey() {
  return "value";
}
function Constructor(object, value = (object[getKey()] ||= new.target)) {
  this.value = value;
}
const object = {};
expect(new Constructor(object).value).toBe(Constructor);
expect(object.value).toBe(Constructor);

let reads = 0;
let writes = 0;
const skip = {
  get value() {
    reads++;
    return 1;
  },
  set value(value) {
    writes++;
  },
};
function getObject() {
  return skip;
}
function shortCircuit(value = (getObject().value ||= (() => { throw new Error(); })())) {
  return value;
}
expect(shortCircuit()).toBe(1);
expect([reads, writes]).toEqual([1, 0]);

function tdz(value = (getObject()[later] ||= 1), later = "value") {}
expect(() => tdz()).toThrow(ReferenceError);
