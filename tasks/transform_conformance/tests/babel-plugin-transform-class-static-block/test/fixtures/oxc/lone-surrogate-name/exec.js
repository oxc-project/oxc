const o = { "\uD800": class { static { this.seen = this.name; } } };
expect([o["\uD800"].name, o["\uD800"].seen]).toEqual(["\uD800", "\uD800"]);
class A {
  static {}
  static "\uDC00" = class { static seen = this.name; };
}
expect([A["\uDC00"].name, A["\uDC00"].seen]).toEqual(["\uDC00", "\uDC00"]);
