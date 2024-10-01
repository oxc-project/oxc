function outer() {
  return () => {
    class C {
      [this]() {}
    }
  };
}

function outer2() {
  class C {
    [this]() {}
  }
}
