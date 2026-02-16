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

// Test no-misused-spread with allow option
// Spreading a function triggers the rule
function allowedFunc() { return 1; }
function notAllowedFunc() { return 2; }

// This should NOT error because allowedFunc is in the allow list
const allowedSpread = { ...allowedFunc };

// This SHOULD error because notAllowedFunc is NOT in the allow list
const notAllowedSpread = { ...notAllowedFunc };

// Test no-unnecessary-type-assertion with checkLiteralConstAssertions option
// When checkLiteralConstAssertions is true, this SHOULD error
const literalConst = 'hello' as const;

// Test no-unsafe-member-access with allowOptionalChaining option
declare const anyValue: any;
// This should NOT error because allowOptionalChaining is true and we use ?.
const optionalAccess = anyValue?.foo;
// This SHOULD error because it's not using optional chaining
const unsafeAccess = anyValue.bar;

// Test no-base-to-string with checkUnknown option
declare const unknownValue: unknown;
// This SHOULD error because checkUnknown is true
const unknownStr = unknownValue.toString();

// Test no-unnecessary-condition with allowConstantLoopConditions option
declare const alwaysTruthyObject: object;
// This SHOULD error because this object is always truthy
if (alwaysTruthyObject) {
  result += 1;
}

// This should NOT error because allowConstantLoopConditions is true
while (true) {
  break;
}

// Test consistent-type-exports with fixMixedExportsWithInlineTypeSpecifier option
type ExportOnlyType = { value: number };
const exportOnlyValue = 1;
// This SHOULD error because ExportOnlyType is only used as a type.
export { ExportOnlyType, exportOnlyValue };

// Test strict-void-return with allowReturnAny option
declare function takesVoidCallback(cb: () => void): void;
declare const anyReturnValue: any;
// This should NOT error because allowReturnAny is true
takesVoidCallback(() => anyReturnValue);
// This SHOULD error because returning string is not allowed in a void callback
takesVoidCallback(() => 'not-void');

// Test prefer-readonly-parameter-types options
function takesAllowedType(input: RegExp): void {
  console.log(input.source);
}

interface MutableParameter {
  value: string;
}

// This SHOULD error because parameter type is mutable
function takesMutableParameter(input: MutableParameter): void {
  console.log(input.value);
}

// Test prefer-readonly with onlyInlineLambdas option
class PreferReadonlyOptionExample {
  private handler = () => 1;
  getValue() {
    return this.handler();
  }
}

// Test prefer-string-starts-ends-with with allowSingleElementEquality option
declare const boundaryText: string;
// This should NOT error because single element equality is allowed
const boundaryCharMatch = boundaryText[0] === 'a';
// This SHOULD error because startsWith is preferred here
const boundarySliceMatch = boundaryText.slice(0, 3) === 'foo';

// Test consistent-return with treatUndefinedAsUnspecified option
function maybeReturnValue(flag: boolean): string | undefined {
  if (flag) {
    return 'value';
  }
  // This SHOULD be treated as an unspecified return with the option enabled
  return undefined;
}

// Test only-throw-error with allowRethrowing option
// When allowRethrowing is false, rethrowing a caught error SHOULD error
try {
  throw new Error('test');
} catch (e) {
  throw e; // This SHOULD error because allowRethrowing is false
}

export { result, customStr, allowedSpread, notAllowedSpread, literalConst, optionalAccess, unsafeAccess, unknownStr };
