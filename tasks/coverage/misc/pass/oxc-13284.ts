// Not sure this is correct, but adding this test to make sure it doesn't panic at least.
// See comment in `check_super` in `oxc_semantic`.

class C {
  [keys: string]: typeof super.foo;
}

class D {
  [keys: string]: typeof import('x', { with: super.foo }).y;
}

class Outer extends Super {
  constructor() {
    class Inner {
      [keys: typeof super.foo]: string;
    }
    class Inner2 {
      [keys: typeof import('x', { with: super.foo }).y]: string;
    }
    class Inner3 {
      [keys: typeof import('x', { with: super() }).y]: string;
    }
  }

  prop1 = class Inner {
    [keys: typeof super.foo]: string;
  };
  prop2 = class Inner {
    [keys: typeof import('x', { with: super.foo }).y]: string;
  };

  accessor access1 = class Inner {
    [keys: typeof super.foo]: string;
  };
  accessor access2 = class Inner {
    [keys: typeof import('x', { with: super.foo }).y]: string;
  };

  method() {
    class Inner {
      [keys: typeof super.foo]: string;
    }
    class Inner2 {
      [keys: typeof import('x', { with: super.foo }).y]: string;
    }
  }
}
