const basic = /(?:(?<year>\d{4})|(?<year>\d{2}))-(?<month>\d{2})/;
const backreference = /(?:(?<name>a)|(?<name>b))\k<name>/;
/(?<test>a)|(?<test>b)/.test("b");
