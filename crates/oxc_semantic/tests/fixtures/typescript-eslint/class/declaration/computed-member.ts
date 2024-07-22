const outer1 = 'a';
const outer2 = 'b';
class A {
  [outer1] = 1;
  [outer2]() {}
}
