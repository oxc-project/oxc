let bound = "A";

class Outer {
  static B = () => {
    // Transform
    super.A++;
    super.A--;
    ++super.A;
    --super.A;

    super[bound]++;
    super[bound]--;
    ++super[bound];
    --super[bound];

    super[unbound]++;
    super[unbound]--;
    ++super[unbound];
    --super[unbound];

    class Inner {
      method() {
        // Don't transform
        super.A++;
        super.A--;
        ++super.A;
        --super.A;

        super[bound]++;
        super[bound]--;
        ++super[bound];
        --super[bound];

        super[unbound]++;
        super[unbound]--;
        ++super[unbound];
        --super[unbound];
      }

      static staticMethod() {
        // Don't transform
        super.A++;
        super.A--;
        ++super.A;
        --super.A;

        super[bound]++;
        super[bound]--;
        ++super[bound];
        --super[bound];

        super[unbound]++;
        super[unbound]--;
        ++super[unbound];
        --super[unbound];
      }
    }
  };
}
