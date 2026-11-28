// A comment after `extends` stays after it, past type parameters too.
class WithTypeParameters<T> extends
// c
B {}
class BeforeKeyword<T>
  // a
  extends B {}
