// `typescript/no-inferrable-types` _should_ be triggered by this line:
const a: number = 5;
// no-constant-binary-expression is a correctness rule and should NOT trigger since correctness is off.
if (a === {}) {
  console.log("foo");
}
