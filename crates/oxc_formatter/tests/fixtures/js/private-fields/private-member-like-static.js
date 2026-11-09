// DIVERGENCES.md#private-member-like-static
class A {
  static #decorator;
  static decorator;
  static #Factory;
  static Factory;
  #privateFieldWithAVeryLongNameThatDoesNotFitOnTheLineEvenAfterBreaking;
  publicFieldWithAVeryLongNameThatDoesNotFitOnTheLineEvenAfterBreakingXX;

  m() {
    foo(someIdentifierName.#privateFieldWithAVeryLongNameThatDoesNotFitOnTheLineEvenAfterBreaking);
    foo(someIdentifierName.publicFieldWithAVeryLongNameThatDoesNotFitOnTheLineEvenAfterBreakingXX);
    this.#Factory.someMethodName(argumentOne).anotherMethodName(argumentTwo).third();
    this.Factory.someMethodName(argumentOneX).anotherMethodName(argumentTwo).third();
  }

  n(obj) {
    const a = obj.#decorator(someArgument)((aaaaaaaaaaaa, bbbbbbbbbbbbbbbb, cccccccccccccccc) => {
      return a;
    });
    const b = obj.decorator(someArgument)((aaaaaaaaaaaa, bbbbbbbbbbbbbbbbb, cccccccccccccccc) => {
      return a;
    });
  }

  @A.#decorator a() {}
  @A.decorator b() {}
}
