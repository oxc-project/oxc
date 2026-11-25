class A {
  @x! a() {}
  @x.y! b() {}
  @x!.y c() {}
  @x!() d() {}
  @x()! e() {}
  @x.y!.z() f() {}
  @g<number>() g() {}
  @x.y<number>() h() {}
  @(g<number>) i() {}
  @(h()<number>) j() {}
  @(x().y) k() {}
  @(x?.y!) l() {}
  @(x["y"]) m() {}
  @(x as any) n() {}
  method(@x p: any, @x.y() q: any, @(x()()) r: any) {}
}

@x! class B {}
@g<number>() class C {}
@(x().y) class D {}

// TypeScript's parser accepts any `LeftHandSideExpression` as a decorator; only its type checker
// reports TS1497 for these. `experimentalDecorators` code relies on them, e.g.
// `@http.controller('hooks').middleware(M)` (oxc-2562), so they are not parse errors here.
class E {
  @x()() a() {}
  @x().y b() {}
  @x().y() c() {}
  @(x)(1) d() {}
  @(x).y e() {}
  @this.x f() {}
}
