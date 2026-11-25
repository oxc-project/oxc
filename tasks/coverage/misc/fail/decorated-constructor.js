class A {
  @dec constructor() {}
}

class B {
  @dec 'constructor'() {}
}

(class {
  @dec @dec2 constructor() {}
});
