// Examples of incorrect code for no-deprecated rule

/** @deprecated Use apiV2 instead. */
function getVersion(): string {
  return "v1";
}

getVersion();