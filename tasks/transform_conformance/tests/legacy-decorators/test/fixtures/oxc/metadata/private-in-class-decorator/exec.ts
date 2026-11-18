import "reflect-metadata";

const calls: string[] = [];
function member(brand: boolean): any {
  expect(brand).toBe(false);
  return () => { calls.push("member"); };
}
function param(...args: any[]) { calls.push("parameter"); }
function replace(Base: any): any {
  calls.push("class");
  return class extends Base { replaced = true; };
}

@replace
class C {
  #brand = 0;
  @member(#brand in C) method() {}
  constructor(@param value: number) {}
  self() { return C; }
}

const instance = new C(1);
expect(calls).toEqual(["member", "parameter", "class"]);
expect((instance as any).replaced).toBe(true);
expect(instance.self()).toBe(C);
expect(Reflect.getMetadata("design:paramtypes", C)).toEqual([Number]);
