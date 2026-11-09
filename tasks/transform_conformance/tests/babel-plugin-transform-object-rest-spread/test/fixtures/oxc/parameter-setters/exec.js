const obj = { set value({ a = 1, b = a, ...rest }) { this.result = b; } };
obj.value = {};
expect(obj.result).toBe(1);
expect(Object.getOwnPropertyDescriptor(obj, 'value').set.length).toBe(1);
class Setters {
  set value({ a = 2, b = a, ...rest }) { this.result = b; }
  static set value([{ a = 3, b = a, ...rest }]) { this.result = b; }
}
const instance = new Setters();
instance.value = {};
expect(instance.result).toBe(2);
expect(Object.getOwnPropertyDescriptor(Setters.prototype, 'value').set.length).toBe(1);
Setters.value = [{}];
expect(Setters.result).toBe(3);
expect(Object.getOwnPropertyDescriptor(Setters, 'value').set.length).toBe(1);
