// The heritage is evaluated in the class scope: the class name there is the inner binding,
// which is initialized with the class decorators' replacement.
function replace(C) {
  return class Replacement extends C {};
}

let getExpression;
const B = @replace class A extends ((getExpression = () => A), class {}) {};
expect(getExpression()).toBe(B);

let getDeclaration;
@replace class D extends ((getDeclaration = () => D), class {}) {}
expect(getDeclaration()).toBe(D);
expect(D.name).toBe("Replacement");

// Reading the binding while the heritage is evaluated is still in its TDZ.
expect(() => {
  @replace class E extends E {}
}).toThrow(ReferenceError);
