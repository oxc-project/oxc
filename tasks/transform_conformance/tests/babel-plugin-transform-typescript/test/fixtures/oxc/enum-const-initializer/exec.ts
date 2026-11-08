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

const a0 = "s";
const a1 = a0;
const a2 = a1;
const a3 = a2;
const a4 = a3;
const a5 = a4;
const a6 = a5;
const a7 = a6;
const a8 = a7;
const a9 = a8;
const a10 = a9;
const a11 = a10;
const a12 = a11;
const a13 = a12;
const a14 = a13;
const a15 = a14;
const a16 = a15;
const a17 = a16;
const a18 = a17;
const a19 = a18;
const a20 = a19;
const a21 = a20;
const a22 = a21;
const a23 = a22;
const a24 = a23;
const a25 = a24;
const a26 = a25;
const a27 = a26;
const a28 = a27;
const a29 = a28;
const a30 = a29;
const a31 = a30;
const a32 = a31;
const a33 = a32;
const a34 = a33;
const a35 = a34;
const a36 = a35;
const a37 = a36;
const a38 = a37;
const a39 = a38;
const a40 = a39;
const a41 = a40;
const a42 = a41;
const a43 = a42;
const a44 = a43;
const a45 = a44;
const a46 = a45;
const a47 = a46;
const a48 = a47;
const a49 = a48;
const a50 = a49;
const a51 = a50;
const a52 = a51;
const a53 = a52;
const a54 = a53;
const a55 = a54;
const a56 = a55;
const a57 = a56;
const a58 = a57;
const a59 = a58;
const a60 = a59;
const a61 = a60;
const a62 = a61;
const a63 = a62;
const a64 = a63;
const a65 = a64;
enum LongAlias { A = a65 }
expect(LongAlias).toEqual({ A: "s" });

const infinity = Infinity;
const nan = NaN;
function nonFinite(Infinity: number, NaN: number) {
  enum Values { Positive = infinity, Negative = -infinity, NotANumber = nan }
  return [Values.Positive, Values.Negative, Values.NotANumber];
}
const values = nonFinite(7, 9);
expect(values[0]).toBe(Infinity);
expect(values[1]).toBe(-Infinity);
expect(values[2]).toBeNaN();
