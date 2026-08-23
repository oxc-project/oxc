const spread = foo(...args) ? value : someExtremelyLongAlternateExpressionThatCannotPossiblyFitOnThisLine;

const ignored = condition ? consequent : // oxfmt-ignore
foo(    1,2,3,4);

const prettierIgnored = condition ? consequent : // prettier-ignore
foo(    1,2,3,4);

const linted = condition ? consequent : // eslint-disable-line no-alert
alert(  "message"  );

const tslinted = condition ? consequent : // tslint:disable-line
alternate;

condition ? /* comment */ bar : someExtremelyLongAlternateExpression;
condition ? bar /* comment */ : someExtremelyLongAlternateExpression;

({ foo: bar }).foo ? short : someExtremelyLongAlternateExpression;
[foo].length ? short : someExtremelyLongAlternateExpression;
foo++ ? short : someExtremelyLongAlternateExpression;
(foo = bar) ? short : someExtremelyLongAlternateExpression;
(foo => bar) ? short : someExtremelyLongAlternateExpression;
foo(bar, baz, qux) ? short : someExtremelyLongAlternateExpression;
