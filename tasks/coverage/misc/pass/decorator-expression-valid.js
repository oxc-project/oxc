class A {
  #p;
  @a a() {}
  @a.b b() {}
  @a.b.c c() {}
  @A.#p d() {}
  @A.#p.q e() {}
  @a() f() {}
  @a.b(1) g() {}
  @A.#p(1, 2) h() {}
  @(a) i() {}
  @(a.b()) j() {}
  @(a().b) k() {}
  @(a()()) l() {}
  @(a, b) n() {}
  @(a[b]) o() {}
  @(a?.b) p() {}
  @(a``) q() {}
  @(this.x) r() {}
  @(new a) s() {}
  @((a)) t() {}
  @a [b]() {}
  @(a) [b]() {}
  @a() [b]() {}
  @a @b.c @d() @(e) u() {}
}

@a class C1 {}
@a.b class C2 {}
@a.b() class C3 {}
@(a) class C4 {}
@(a(b)) class C5 {}

