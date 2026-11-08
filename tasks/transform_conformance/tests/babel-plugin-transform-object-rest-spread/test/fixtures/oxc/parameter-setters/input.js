const obj = { set value({ a = 1, b = a, ...rest }) { this.result = b; } };
class Setters {
  set value({ a = 2, b = a, ...rest }) { this.result = b; }
  static set value([{ a = 3, b = a, ...rest }]) { this.result = b; }
}
