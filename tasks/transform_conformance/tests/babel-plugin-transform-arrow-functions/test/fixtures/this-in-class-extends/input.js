function outer() {
  return () => {
    class C extends this {}
  };
}

function outer2() {
  class C extends this {}
}
