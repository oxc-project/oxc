class A {
  @dec constructor() {}
}

class B {
  @dec 'constructor'() {}
}

class C {
  @dec \u0063onstructor() {}
}

(class {
  @dec @dec2 constructor() {}
});
