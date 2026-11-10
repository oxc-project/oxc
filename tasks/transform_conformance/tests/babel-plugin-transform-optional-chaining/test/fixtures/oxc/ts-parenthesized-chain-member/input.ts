function test(a, k) {
  (a?.b)!.c?.d;
  (a?.b)![k]?.c;
  (a?.b)!()?.c;
  (a?.b)!.c?.();
  (a?.b)!?.c;
  ((a?.b)!)(k())?.c;
  ((a?.())!)();
}
