class F {
  x = class {} as any;
  y = (() => {}) satisfies Function;
  z = <any>function () {};
  [k] = (class {})!;
}
