// Member accesses are inlined; the bare value reference forces the IIFE
// to be kept so the runtime binding still exists.
const enum Phase {
  one = "one",
  two = "two",
}

const a = Phase.one;
const b = Phase.two;
const c = Phase;
