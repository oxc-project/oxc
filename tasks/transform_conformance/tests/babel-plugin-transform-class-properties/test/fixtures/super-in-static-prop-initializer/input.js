class C {
  static prop = () => {
    // Transform
    super.prop;
    super[prop];
    super.prop();
    super[prop]();

    const obj = {
      method() {
        // Don't transform
        super.prop;
        super[prop];
        super.prop();
        super[prop]();
      }
    };

    class Inner {
      method() {
        // Don't transform
        super.prop;
        super[prop];
        super.prop();
        super[prop]();
      }

      static staticMethod() {
        // Don't transform
        super.prop;
        super[prop];
        super.prop();
        super[prop]();
      }
    }
  };
}
