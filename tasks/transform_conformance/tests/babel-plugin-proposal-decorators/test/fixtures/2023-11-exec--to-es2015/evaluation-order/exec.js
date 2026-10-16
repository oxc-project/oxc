const log = [];
const dec = (n) => {
  log.push(n);
  return () => {};
};
const key = (n) => {
  log.push(n);
  return "k" + n;
};

@dec(0)
class Foo extends (log.push(1), Object) {
  @dec(2) [key(3)]() {}
  @dec(4) static [key(5)] = 1;
  @dec(6) [key(7)];
  @dec(8) accessor [key(9)];
  @dec(10) get [key(11)]() { return 1; }
}

expect(log).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
expect(new Foo().k7).toBe(undefined);
expect(Foo.k5).toBe(1);
