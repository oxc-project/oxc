export class ReadonlyLiteralInitializers {
  readonly directTrue = true;
  readonly parenTrue = (true);
  readonly constAssertTrue = true as const;
  readonly nestedConstAssert = ((true as const));

  readonly directString = "x";
  readonly parenString = ("x");
  readonly templateNoExpr = `x`;
  readonly constAssertString = ("x" as const);

  readonly unaryNumber = -1;
  readonly unaryParenNumber = (-1);
  readonly constAssertNumber = (1 as const);

  writableTrue = true;
}
