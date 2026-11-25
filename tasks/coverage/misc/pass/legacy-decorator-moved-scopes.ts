class A {
  constructor(@dec((x: A) => x) a: any) {}
  method(@dec(() => {}) p: any, @(((t: any, k: any, i: number) => {})) q: any) {}
  @(<T>(t: T) => t) prop = 1;
}

@dec(() => {})
class B {
  constructor(@dec(() => {}) b: any) {}
}
