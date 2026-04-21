o = {
  foo() {
    super.foo();
    return () => super.foo();
  },

  get bar() {
    const f = () => super.bar;
    return super.bar;
  },

  set bar(v) {
    const f = () => super.bar;
    super.bar = v;
  },
};
