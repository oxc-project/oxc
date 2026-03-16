const tuple = [1, 2] as const;
const n = 3;

declare function fn(): number;

export class ReadonlyClassPropertyVariations {
  readonly emptyArray = [];
  readonly emptyArrayAsConst = [] as const;
  readonly emptyArrayWithTypeAnnotation: number[] = [];
  readonly emptyArrayAsTypeAssertion = [] as number[];
  readonly emptyArrayAsConstTypeAssertion = <const>[];
  readonly nestedEmptyArrayAsConst = (([] as const));

  readonly arrayWithNumberLiterals = [1, 2, 3];
  readonly arrayWithMixedLiterals = [1, "x", true, null, undefined];
  readonly arrayWithElision = [, 1, , 2];
  readonly arrayWithOnlyElision = [,,];
  readonly arrayAsConst = [1, "x", true] as const;
  readonly nestedArray = [[1], [] as const, [1, 2] as const];
  readonly arrayWithObjectLiterals = [{ a: 1 }, { b: "x" as const }];
  readonly arrayWithUnaryNumberLiterals = [-1, +2];
  readonly arrayWithNoExprTemplateLiteral = [`x`];
  readonly arrayWithUndefinedIdentifier = [undefined];
  readonly arrayWithConstAssertedElements = [({ a: 1 } as const), ([1, 2] as const)];

  readonly arrayWithSpreadConstTuple = [...tuple];
  readonly arrayWithSpreadLiteral = [...[1, 2]];
  readonly arrayWithIdentifierElement = [tuple];
  readonly arrayWithCallElement = [fn()];
  readonly arrayWithTemplateExprElement = [`x${n}`];
  readonly arrayWithInvalidUnaryElement = [+"x"];
  readonly arraySatisfies = [] satisfies number[];
  readonly arrayNonNull = []!;
  readonly invalidDirectUnary = +"x";
  readonly invalidTemplateLiteral = `x${n}`;

  readonly annotatedSpread: readonly number[] = [...tuple];
  readonly annotatedIdentifier: readonly [1, 2] = tuple;
  readonly annotatedCall: unknown[] = [fn()];
  readonly annotatedTemplateExpr: string[] = [`x${n}`];
}
