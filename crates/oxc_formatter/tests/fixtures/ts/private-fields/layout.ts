// `a.#b` (class A) is laid out the same as `a._b` (class B)
class A {
  #b;
  m() {
    (someVeryLongExpressionName as unknown as SomeVeryLongTypeName<WithSomeTypeArgumentsss>).#b;
    (someVeryLongExpressionName satisfies SomeVeryLongTypeName<WithSomeTypeArgumentsssssss>).#b;
    (<SomeVeryLongTypeName<WithSomeTypeArguments>>someVeryLongExpressionNameeeeeeeeeeeeeeeeeee).#b;
  }
}

class B {
  _b;
  m() {
    (someVeryLongExpressionName as unknown as SomeVeryLongTypeName<WithSomeTypeArgumentsss>)._b;
    (someVeryLongExpressionName satisfies SomeVeryLongTypeName<WithSomeTypeArgumentsssssss>)._b;
    (<SomeVeryLongTypeName<WithSomeTypeArguments>>someVeryLongExpressionNameeeeeeeeeeeeeeeeeee)._b;
  }
}
