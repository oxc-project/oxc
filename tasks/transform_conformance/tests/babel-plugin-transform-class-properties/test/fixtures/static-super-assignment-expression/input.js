let bound = "A";

class Outer {
  static B = () => {
    // Transform
    super.A = 1;
    super.A += 1;
    super.A -= 1;
    super.A &&= 1;
    super.A ||= 1;

    super[bound] = 1;
    super[bound] += 1;
    super[bound] -= 1;
    super[bound] &&= 1;
    super[bound] ||= 1;

    super[unbound] = 1;
    super[unbound] += 1;
    super[unbound] -= 1;
    super[unbound] &&= 1;
    super[unbound] ||= 1;

    class Inner {
      method() {
        // Don't transform
        super.A = 1;
        super.A += 1;
        super.A -= 1;
        super.A &&= 1;
        super.A ||= 1;

        super[bound] = 1;
        super[bound] += 1;
        super[bound] -= 1;
        super[bound] &&= 1;
        super[bound] ||= 1;

        super[unbound] = 1;
        super[unbound] += 1;
        super[unbound] -= 1;
        super[unbound] &&= 1;
        super[unbound] ||= 1;
      }

      static staticMethod() {
        // Don't transform
        super.A = 1;
        super.A += 1;
        super.A -= 1;
        super.A &&= 1;
        super.A ||= 1;

        super[bound] = 1;
        super[bound] += 1;
        super[bound] -= 1;
        super[bound] &&= 1;
        super[bound] ||= 1;

        super[unbound] = 1;
        super[unbound] += 1;
        super[unbound] -= 1;
        super[unbound] &&= 1;
        super[unbound] ||= 1;
      }
    }
  };
}
