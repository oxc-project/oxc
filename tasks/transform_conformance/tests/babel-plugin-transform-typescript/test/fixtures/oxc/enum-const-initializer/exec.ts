const o = "o";
const alias = o + "k";
enum E { A = o, B = 1, C = alias }
expect(JSON.stringify(E)).toBe('{"1":"B","A":"o","B":1,"C":"ok"}');

{
  const o = "inner";
  enum Inner { A = o }
  expect(Inner).toEqual({ A: "inner" });
}

const surrogate = "\ud800";
const surrogateAlias = surrogate;
enum Surrogate { A = surrogateAlias, B = A }
expect(Surrogate).toEqual({ A: "\ud800", B: "\ud800" });

let mutable = "before";
mutable = "after";
enum Mutable { A = mutable }
expect(Mutable).toEqual({ A: "after", after: "A" });

const a = "outer";
function f() {
  function g() {
    const b = a;
    enum Nested { X = b }
    return Nested.X;
  }
  const a = "inner";
  return g();
}
expect(f()).toBe("inner");

const s0 = "x";
const s1 = s0 + s0;
const s2 = s1 + s1;
const s3 = s2 + s2;
const s4 = s3 + s3;
const s5 = s4 + s4;
const s6 = s5 + s5;
const s7 = s6 + s6;
const s8 = s7 + s7;
const s9 = s8 + s8;
const s10 = s9 + s9;
const s11 = s10 + s10;
const s12 = s11 + s11;
enum Large { A = s12 }
expect(Object.keys(Large)).toEqual(["A"]);
expect(Large.A.length).toBe(4096);
