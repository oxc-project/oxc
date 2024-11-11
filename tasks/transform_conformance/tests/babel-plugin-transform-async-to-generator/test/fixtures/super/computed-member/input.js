class Foo extends class {} {
  async method() {
    super['name'];
    {
      super['name']();
      super['object']['name']();
    }
  }
}
