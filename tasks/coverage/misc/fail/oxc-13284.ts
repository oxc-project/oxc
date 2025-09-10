// Not sure this is correct, but adding this test to make sure it doesn't panic at least.
// See comment in `check_super` in `oxc_semantic`.

// `super()`
class C extends Super {
  [keys: string]: typeof import('x', { with: super() }).y;
}

class D extends Super {
  [keys: typeof import('x', { with: super() }).y]: string;
}

class Outer extends Super {
  constructor() {
    class Inner {
      [keys: string]: typeof import('x', { with: super() }).y;
    }
  }
}

class Outer2 {
  constructor() {
    class Inner extends Super {
      [keys: typeof import('x', { with: super() }).y]: string;
    }
  }
}

// `super.foo`
class E {
  [keys: typeof super.foo]: string;
}

class F {
  [keys: typeof import('x', { with: super.foo }).y]: string;
}
