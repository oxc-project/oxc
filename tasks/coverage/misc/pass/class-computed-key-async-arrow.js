class C {
  [async x => x]() {}
  static [async x => x]() {}
  [async x => x] = 1;
  @dec accessor [async x => x] = 1;
}

(class {
  static [async x => x];
});
