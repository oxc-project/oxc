class C {
  foo() {
    return {
      bar() {
        super.bar();
      },
    };
  }
}
