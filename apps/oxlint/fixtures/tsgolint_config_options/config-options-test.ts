// Test file for exercising all tsgolint rule options

// typescript/no-floating-promises - with various options
async function testNoFloatingPromises() {
  // Should error: floating promise without void
  Promise.resolve(42);
  
  // Should work with IIFE (ignoreIIFE: true)
  (async () => { await Promise.resolve(); })();
  
  // Should error: ignoreVoid is false
  void Promise.resolve();
  
  // Thenable check (checkThenables: true)
  const thenable = { then: (cb: any) => cb() };
  thenable.then(() => {});
}

// typescript/no-misused-promises - all check options
async function testNoMisusedPromises() {
  const promise = Promise.resolve(true);
  
  // checksConditionals
  if (promise) { /* should error */ }
  
  // checksSpreads
  const arr = [...promise]; // should error
  
  // checksVoidReturn (all sub-options)
  function takesVoid(fn: () => void) {}
  takesVoid(async () => 42); // should error
}

// typescript/strict-boolean-expressions - various allow options
function testStrictBooleanExpressions(
  any: any,
  nullableBoolean: boolean | null,
  nullableNumber: number | null,
  nullableString: string | null,
  nullableEnum: "a" | "b" | null,
  nullableObject: object | null,
  str: string,
  num: number
) {
  // allowAny: true - should pass
  if (any) {}
  
  // allowNullableBoolean: true - should pass
  if (nullableBoolean) {}
  
  // allowNullableNumber: true - should pass
  if (nullableNumber) {}
  
  // allowNullableString: true - should pass
  if (nullableString) {}
  
  // allowNullableEnum: true - should pass
  if (nullableEnum) {}
  
  // allowNullableObject: false - should error
  if (nullableObject) {}
  
  // allowString: false - should error
  if (str) {}
  
  // allowNumber: false - should error
  if (num) {}
}

// typescript/restrict-template-expressions
function testRestrictTemplateExpressions(
  any: any,
  arr: any[],
  bool: boolean,
  nullish: null | undefined,
  num: number,
  regex: RegExp,
  neverVal: never
) {
  // allowAny: false - should error
  `${any}`;
  
  // allowArray: true - should pass
  `${arr}`;
  
  // allowBoolean: false - should error
  `${bool}`;
  
  // allowNullish: false - should error
  `${nullish}`;
  
  // allowNumber: false - should error
  `${num}`;
  
  // allowRegExp: false - should error
  `${regex}`;
  
  // allowNever: true - should pass (though unreachable)
  // `${neverVal}`;
  
  // allow: Error, URL from lib - should pass
  `${new Error("test")}`;
  `${new URL("http://example.com")}`;
}

// typescript/restrict-plus-operands
function testRestrictPlusOperands(
  any: any,
  bool: boolean,
  nullish: null | undefined,
  str: string,
  num: number,
  regex: RegExp
) {
  // allowAny: false - should error
  const r1 = any + 1;
  
  // allowBoolean: false - should error
  const r2 = bool + 1;
  
  // allowNullish: false - should error
  const r3 = nullish + 1;
  
  // allowNumberAndString: false - should error
  const r4 = num + str;
  
  // allowRegExp: false - should error
  const r5 = regex + "test";
  
  // skipCompoundAssignments: true - should pass
  let x = 0;
  x += any;
}

// typescript/no-unnecessary-boolean-literal-compare
function testNoUnnecessaryBooleanLiteralCompare(bool: boolean | null) {
  // allowComparingNullableBooleansToFalse: false - should error
  if (bool === false) {}
  
  // allowComparingNullableBooleansToTrue: false - should error
  if (bool === true) {}
}

// typescript/no-confusing-void-expression
function testNoConfusingVoidExpression() {
  function returnsVoid(): void {}
  
  // ignoreArrowShorthand: true - should pass
  const arrow = () => returnsVoid();
  
  // ignoreVoidOperator: true - should pass
  const x = void returnsVoid();
  
  // ignoreVoidReturningFunctions: true - should pass
  const y = returnsVoid();
}

// typescript/prefer-promise-reject-errors
async function testPreferPromiseRejectErrors(any: any, unknown: unknown) {
  // allowEmptyReject: true - should pass
  Promise.reject();
  
  // allowThrowingAny: false - should error
  Promise.reject(any);
  
  // allowThrowingUnknown: false - should error
  Promise.reject(unknown);
  
  // Should error: not an Error
  Promise.reject("string");
}

// typescript/no-unnecessary-type-assertion
type Foo = { foo: string };
type Bar = { bar: number };

function testNoUnnecessaryTypeAssertion() {
  const str: string = "hello";
  
  // typesToIgnore: ["Foo", "Bar"] - should pass
  const foo = str as unknown as Foo;
  const bar = str as unknown as Bar;
  
  // Should error: not in typesToIgnore
  const num = 42 as number;
}

// typescript/promise-function-async
type SpecialPromise<T> = Promise<T>;
type CustomThenable<T> = { then: (cb: (val: T) => void) => void };

// allowAny: false - should error
function returnsAny(): any {
  return Promise.resolve();
}

// allowedPromiseNames: ["SpecialPromise", "CustomThenable"] - should pass
function returnsSpecialPromise(): SpecialPromise<number> {
  return Promise.resolve(42);
}

function returnsCustomThenable(): CustomThenable<number> {
  return { then: (cb) => cb(42) };
}

// checkArrowFunctions: true - should error
const arrowReturnsPromise = (): Promise<void> => Promise.resolve();

// checkFunctionDeclarations: true - should error
function declReturnsPromise(): Promise<void> {
  return Promise.resolve();
}

// checkFunctionExpressions: true - should error
const exprReturnsPromise = function(): Promise<void> {
  return Promise.resolve();
};

class TestClass {
  // checkMethodDeclarations: true - should error
  methodReturnsPromise(): Promise<void> {
    return Promise.resolve();
  }
}

// typescript/no-duplicate-type-constituents
// ignoreIntersections: false - should error
type DupIntersection = string & string;

// ignoreUnions: false - should error
type DupUnion = string | string;

// typescript/switch-exhaustiveness-check
type Status = "pending" | "success" | "error";

function testSwitchExhaustivenessCheck(status: Status, nonUnion: string) {
  // allowDefaultCaseForExhaustiveSwitch: true - should pass
  switch (status) {
    case "pending":
    case "success":
    case "error":
    default:
      break;
  }
  
  // defaultCaseCommentPattern: "@skip-exhaustive-check" - should pass
  switch (status) {
    case "pending":
    default: // @skip-exhaustive-check
      break;
  }
  
  // requireDefaultForNonUnion: true - should error (no default)
  switch (nonUnion) {
    case "a":
      break;
  }
}

// typescript/unbound-method
class TestUnboundMethod {
  static staticMethod() {}
  instanceMethod() {}
}

function testUnboundMethod() {
  // ignoreStatic: true - should pass
  const fn1 = TestUnboundMethod.staticMethod;
  
  // Should error: instance method unbound
  const obj = new TestUnboundMethod();
  const fn2 = obj.instanceMethod;
}

// typescript/only-throw-error
class CustomError extends Error {}

async function testOnlyThrowError(any: any, unknown: unknown) {
  // allow: CustomError - should pass
  throw new CustomError();
  
  // allowThrowingAny: false - should error
  throw any;
  
  // allowThrowingUnknown: false - should error
  throw unknown;
  
  // Should error: not an Error
  throw "string";
  throw 42;
}

// typescript/return-await - "always"
async function testReturnAwait() {
  // Should error: missing await (configured as "always")
  return Promise.resolve(42);
  
  // Should pass
  return await Promise.resolve(42);
  
  try {
    return await Promise.resolve(42); // should pass
  } catch (e) {
    throw e;
  }
}

export {};
