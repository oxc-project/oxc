// Every line deviates from Prettier (DIVERGENCES.md#instantiation-expression-parens).
const a = (make<T>)!;
(make<T>)!.b;
(make<T>)!();
export default (class {})<string>;
export default (function () {})<string>;
export default (class {})<string> as A;
(class {})<string>;
(function () {})<string>;
