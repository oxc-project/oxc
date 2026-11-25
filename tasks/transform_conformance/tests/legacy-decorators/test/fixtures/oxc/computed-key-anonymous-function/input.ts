function dec() {
  return () => {};
}

class C {
  @dec()
  [() => {}]() {}
}
