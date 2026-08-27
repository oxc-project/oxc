const basic = /(?:(?<year>\d{4})|(?<year>\d{2}))-(?<month>\d{2})/;
const backreference = /(?:(?<name>a)|(?<name>b))\k<name>/;
const escapedBackreference = /(?<a>x)|(?<\u0061>y)\k<\u0061>/;
/(?<test>a)|(?<test>b)/.test("b");
const test = /(?<test>a)|(?<test>b)/.test;
const proto = /(?<__proto__>a)|(?<__proto__>b)/;
