//      10        20        30        40        50        60        70        80
// 45678901234567890123456789012345678901234567890123456789012345678901234567890

// DIVERGENCES.md#arrow-exact-fill-signature

// Conditional body: diverges at printWidth 80
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLengthOkOkOkO = (x) =>
  cond ? longlonglong : shortshort;
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLength = (x) => (y) =>
  cond ? longlonglong : shortshort;
vi.mocked(useCommandStore().formatKeySequenceXY).mockImplementation((command) =>
  cond ? longlonglong : shortshort,
);

// Call body: Prettier keeps the signature too
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLengthOkOkOkO = (x) =>
  callSomething(longlonglong, shortshort);
const exactlyFillsThePrintWidthEightySignatureLinePaddedToLength = (x) => (y) =>
  callSomething(longlonglong, shortshort);
vi.mocked(useCommandStore().formatKeySequenceXY).mockImplementation((command) =>
  callSomething(longlonglong, shortshort),
);
