class A {
  static C = ((() => {})(), { "C": class {
    static seen = this.name;
  } }["C"]);
  static Custom = ((() => {})(), { "Custom": class {
    static name = "custom";
  } }["Custom"]);
}
