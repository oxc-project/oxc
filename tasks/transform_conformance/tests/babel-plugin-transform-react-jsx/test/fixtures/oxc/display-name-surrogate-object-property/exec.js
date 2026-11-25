const React = { createClass: (value) => value };
const obj = {
  "\uD800": React.createClass({}),
  "\uDC00": React.createClass({}),
  [`a\uD800b`]: React.createClass({}),
  "𐀀": React.createClass({}),
  ["\uD800"]: React.createClass({}),
};
for (const name of Object.keys(obj)) {
  expect(obj[name].displayName).toBe(name);
}
