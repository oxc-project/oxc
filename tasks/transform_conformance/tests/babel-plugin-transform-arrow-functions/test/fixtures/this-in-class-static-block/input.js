function outer() {
  return () => {
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
