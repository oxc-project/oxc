const basic = /*#__PURE__*/ babelHelpers.wrapRegExp(/(?:(\d{4})|(\d{2}))-(\d{2})/, {
  year: [1, 2],
  month: 3,
});
const backreference = /*#__PURE__*/ babelHelpers.wrapRegExp(/(?:(a)|(b))\1\2/, {
  name: [1, 2],
});
const escapedBackreference = /*#__PURE__*/ babelHelpers.wrapRegExp(/(x)|(y)\1\2/, {
  a: [1, 2],
});
/(a)|(b)/.test("b");
const test = /(a)|(b)/.test;
const proto = /*#__PURE__*/ babelHelpers.wrapRegExp(/(a)|(b)/, {
  ["__proto__"]: [1, 2],
});
