const outer = 1;
type Outer = 1;
const A = class {
  constructor(
    private a,
    private b = 1,
    private c = a,
    public d = outer,
    public e,
    readonly f: Outer,
  ) {
    a;
  }
};

const unresolved = e;
