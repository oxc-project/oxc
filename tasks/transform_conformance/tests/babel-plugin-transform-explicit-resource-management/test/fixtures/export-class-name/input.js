using x = foo();

export class C {
  static getSelf() { return C; }
}
const K = C;
C = 123;

assert(K.getSelf() === K);
