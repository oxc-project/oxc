class A {
  static {}
  static C = class { static seen = this.name; };
  static {}
  static Custom = class { static name = "custom"; };
}
