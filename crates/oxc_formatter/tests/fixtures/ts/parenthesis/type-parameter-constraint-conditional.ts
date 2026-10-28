// Conditional type in a type parameter's constraint position requires parentheses;
// bare `T extends A extends B ? C : D` is a syntax error.
type F<T extends (A extends B ? C : D)> = T;
// The default position does not require them.
type G<T = A extends B ? C : D> = T;
type H<T extends X, U extends (T extends Y ? P : Q) = T extends Z ? R : S> = U;
const f = <T extends (A extends B ? C : D)>() => true;
