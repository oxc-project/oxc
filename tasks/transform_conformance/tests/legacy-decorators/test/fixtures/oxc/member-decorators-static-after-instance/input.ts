function dec(name: string): any {
  return () => {};
}

class A {
  @dec("instance-prop") prop = 1;
  @dec("static-prop") static s = 2;
  @dec("instance-method") m() {}
  @dec("static-method") static sm() {}
  @dec("instance-accessor") get g() { return 1; }
  @dec("static-accessor") static get sg() { return 1; }
}
