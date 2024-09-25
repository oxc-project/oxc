function outer() {
  return () => {
    class C {
      [this] = 1;
    }
  };
}

function outer2() {
  class C {
    [this] = 1;
  }
}
