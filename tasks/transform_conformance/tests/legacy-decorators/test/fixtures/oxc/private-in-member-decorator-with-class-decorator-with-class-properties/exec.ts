const log: string[] = [];
const originals: any[] = [];

function classDec(Class: any) {
  log.push("class");
  originals.push(Class);
  return class extends Class {};
}
function paramDec() {
  log.push("param");
}
function memberDec(seesOriginalClass: boolean) {
  return () => {
    log.push(`member ${seesOriginalClass}`);
  };
}

@classDec
class A {
  static #a = 1;
  @memberDec(#a in A) m() {}
}

@classDec
class B {
  static #b = 1;
  constructor(@paramDec p?: unknown) {}
  @memberDec(#b in B) m() {}
}

// Member decorators see the original class, then the class decorators run and replace it.
expect(log).toEqual(["member true", "class", "member true", "param", "class"]);
expect(Object.getPrototypeOf(A)).toBe(originals[0]);
expect(Object.getPrototypeOf(B)).toBe(originals[1]);
