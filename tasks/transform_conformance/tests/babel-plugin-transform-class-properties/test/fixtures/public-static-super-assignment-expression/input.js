const ident = "A";
class Outer {
  static A = 0;
  static B = () => {
    super.A += 1;
    super.A -= 1;
    super.A &&= 1;
    super.A ||= 1;
    super.A = 1;

    super[ident] += 1;
    super[ident] -= 1;
    super[ident] &&= 1;
    super[ident] ||= 1;
    super[ident] = 1;
  };
}
