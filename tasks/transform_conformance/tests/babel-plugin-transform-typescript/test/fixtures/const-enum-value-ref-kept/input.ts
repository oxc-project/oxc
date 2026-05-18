// `export default <enum>` is a bare value reference that can't be inlined.
// The IIFE form must be emitted so the binding still exists at runtime,
// otherwise the export resolves to `undefined`/ReferenceError.
const enum Phase {
  one = "one",
  two = "two",
}

export default Phase;
