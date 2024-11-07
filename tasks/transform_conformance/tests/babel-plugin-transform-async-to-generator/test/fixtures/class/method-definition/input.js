class ClassWithAsyncMethods {
  async with_parameters(p, [p1, p2]) {
    return console.log(p, p1, p2);
  }

  async without_parameters() {
    console.log(ClassWithAsyncMethods);
  }
}