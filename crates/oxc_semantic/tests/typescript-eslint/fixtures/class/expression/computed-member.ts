const outer1 = 'a';
const outer2 = 'b';
const A = class {
  [outer1] = 1;
  [outer2]() {}
};
