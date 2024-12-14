class Child extends Parent {
  constructor() {
    super();
    this.scopedFunctionWithThis = () => {
      this.name = {};
    };
  }
}
