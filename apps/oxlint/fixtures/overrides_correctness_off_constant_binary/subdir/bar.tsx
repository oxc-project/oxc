// no-constant-binary-expression is a correctness rule and should NOT trigger since correctness is off.
const a = 5;
if (a === {}) {
  console.log("foo");
}
