const o = { "\uD800": class { static { this.seen = this.name; } } };
class A {
  static {}
  static "\uDC00" = class { static seen = this.name; };
}
