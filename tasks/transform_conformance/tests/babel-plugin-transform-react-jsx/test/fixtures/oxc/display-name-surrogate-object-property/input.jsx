const obj = {
  "\uD800": React.createClass({}),
  "\uDC00": React.createClass({}),
  [`a\uD800b`]: React.createClass({}),
  "𐀀": React.createClass({}),
  ["\uD800"]: React.createClass({}),
};
