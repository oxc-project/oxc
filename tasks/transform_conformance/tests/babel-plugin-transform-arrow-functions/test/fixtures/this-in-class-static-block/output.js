function outer() {
  return function() {
    class C {
      static {
        t = this;
      }
    }
  };
}

function outer2() {
  class C {
    static {
      t = this;
    }
  }
}
