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

  readonly directObject = { a: 1 };
  readonly parenObject = ({ a: 1 });
  readonly constAssertObject = ({ a: 1 } as const);
  readonly nestedConstAssertObject = ((({ a: 1 } as const)));

  readonly directNestedObject = { nested: { a: 1 } };
  readonly constAssertNestedObject = ({ nested: { a: 1 } } as const);

  writableObject = { a: 1 };
  writableTrue = true;
}
