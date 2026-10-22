// DIVERGENCES.md#binary-keyword-class-field-boundary
// DIVERGENCES.md#suppressed-terminator-per-semi
// DIVERGENCES.md#trailing-member-suppression-before-semicolon
class SuppressedIdentifiers {
  "not-valid";
  x = 1;
  // prettier-ignore
  in;
  y = 1;
  // prettier-ignore
  instanceof() {}
}
class InlineIgnore {
  "not-valid";
  x = 1; // prettier-ignore
  in;
}
class TrailingIgnore {
  "not-valid";
  x = 1;
  in // prettier-ignore
  ;
  y = 1;
  instanceof() {} // prettier-ignore
}
class SuppressedStrings {
  "not-valid";
  x = 1;
  // prettier-ignore
  "in";
  y = 1;
  // prettier-ignore
  "instanceof"() {}
}
class LeadingCommentRun {
  "not-valid";
  x = 1;
  /* leading */ /* prettier-ignore */
  in;
}
