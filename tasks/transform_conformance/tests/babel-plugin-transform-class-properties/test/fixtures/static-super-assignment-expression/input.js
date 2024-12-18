const ident = "A";

class Outer {
  static B = () => {
    // Transform
    super.A = 1;
    super.A += 1;
    super.A -= 1;
    super.A &&= 1;
    super.A ||= 1;

    super[ident] = 1;
    super[ident] += 1;
    super[ident] -= 1;
    super[ident] &&= 1;
    super[ident] ||= 1;

    class Inner {
      method() {
        // Don't transform
        super.A = 1;
        super.A += 1;
        super.A -= 1;
        super.A &&= 1;
        super.A ||= 1;

        super[ident] = 1;
        super[ident] += 1;
        super[ident] -= 1;
        super[ident] &&= 1;
        super[ident] ||= 1;
      }

      static staticMethod() {
        // Don't transform
        super.A = 1;
        super.A += 1;
        super.A -= 1;
        super.A &&= 1;
        super.A ||= 1;

        super[ident] = 1;
        super[ident] += 1;
        super[ident] -= 1;
        super[ident] &&= 1;
        super[ident] ||= 1;
      }
    }
  };
}
