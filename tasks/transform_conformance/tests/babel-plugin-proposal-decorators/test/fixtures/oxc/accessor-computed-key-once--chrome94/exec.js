// An auto-accessor's key is converted once, so its getter and setter share it.
let count = 0;
const key = { toString() { return "k" + ++count; } };

class Plain {
  accessor [key] = 1;
  static accessor [key] = 2;
}
expect(count).toBe(2);
expect(Object.getOwnPropertyNames(Plain.prototype)).toEqual(["constructor", "k1"]);
const plain = new Plain();
plain.k1 = 5;
expect(plain.k1).toBe(5);
Plain.k2 = 6;
expect(Plain.k2).toBe(6);
