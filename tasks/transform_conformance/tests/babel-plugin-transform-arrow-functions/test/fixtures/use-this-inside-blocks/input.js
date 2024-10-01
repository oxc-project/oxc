function foo() {
  { let f = () => this; }
  { let f2 = () => this; }
}
