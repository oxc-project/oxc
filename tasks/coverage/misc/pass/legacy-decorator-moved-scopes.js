// The legacy decorator transform moves decorator expressions of class elements out of the class.
// Their scopes must move with them, and lose strict mode in a script.
class A {
  @((t, k) => {}) method() {}
  @((t, k) => {}) static staticMethod() {}
  @(function (t, k) { return function () {}; }) prop = 1;
  @(function (t, k) { "use strict"; return () => {}; }) static staticProp = 2;
  @((t, k) => class { m() {} }) accessor acc = 3;
  @(() => { { let x; } }) get getter() { return 1; }
}

@((c) => c)
class B {
  @((t, k) => {}) method() {}
}

// A private-in expression keeps the decorations in a static block inside the class.
class C {
  #p;
  @(#p in C ? (t, k) => {} : (t, k) => {}) method() {}
}
