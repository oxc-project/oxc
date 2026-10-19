// Every line deviates from Prettier (DIVERGENCES.md#instantiation-expression-parens).
const a = (make<T>)!;
(make<T>)!.b;
(make<T>)!();
(make<T>)<U>;
(make<T>)<U>();
new (make<T>)<U>();
(make<T>)<U>``;
x = (make<T>) + y;
x = (make<T>) >> y;
x = (a + make<T>) - y;
x = (typeof make<T>) + y;
export default (class {})<string>;
export default (function () {})<string>;
export default (class {})<string> as A;
(class {})<string>;
(function () {})<string>;
