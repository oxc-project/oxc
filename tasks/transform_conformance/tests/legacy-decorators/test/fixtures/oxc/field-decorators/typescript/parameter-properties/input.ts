class A {
  @dec a: number;
  @dec b = 1;
  c: number;
  constructor(public p: number, private q = 2) {}
}
class B extends Base {
  @dec a: number;
  constructor(public p: number) {
    super();
  }
}
