class A {
  @dec a: number;
  @dec b: string = "";
  @dec static c: A;
  @dec [k]: boolean;
  constructor(public p: number, @dec q: string) {}
}
