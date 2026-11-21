const calls: string[] = [];
function dec(name: string): any {
  return () => { calls.push(name); };
}

@dec("class")
class A {
  @dec("instance-prop") prop = 1;
  @dec("static-prop") static s = 2;
  @dec("instance-method") m() {}
  @dec("static-method") static sm() {}
  @dec("instance-accessor") get g() { return 1; }
  @dec("static-accessor") static get sg() { return 1; }
  constructor(@dec("constructor-param") value: any) {}
}

class StaticOnly {
  @dec("static-only") static prop = 1;
}

expect(calls).toEqual([
  "instance-prop",
  "instance-method",
  "instance-accessor",
  "static-prop",
  "static-method",
  "static-accessor",
  "constructor-param",
  "class",
  "static-only",
]);
