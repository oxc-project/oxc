type Day = 'Monday' | 'Tuesday';

declare const day: Day;
let result = 0;

// This should NOT error when considerDefaultExhaustiveForUnions is true
// because the default case makes it exhaustive
switch (day) {
  case 'Monday':
    result = 1;
    break;
  default:
    result = 3;
    break;
}

// Test no-base-to-string with ignoredTypeNames option
// CustomStringifiable is in the ignoredTypeNames list, so this should NOT error
declare class CustomStringifiable {
  value: string;
}
declare const custom: CustomStringifiable;
const customStr = custom.toString();

// Test no-deprecated with allow option
/** @deprecated Use newFunction instead */
function allowedDeprecated(): void {}

/** @deprecated Use anotherNewFunction instead */
function notAllowedDeprecated(): void {}

// This should NOT error because allowedDeprecated is in the allow list
allowedDeprecated();

// This SHOULD error because notAllowedDeprecated is NOT in the allow list
notAllowedDeprecated();

export { result, customStr };
