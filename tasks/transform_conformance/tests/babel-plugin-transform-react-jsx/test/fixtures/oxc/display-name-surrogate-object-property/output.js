const obj = {
  "\ud800": React.createClass({ displayName: "\ud800" }),
  "\udc00": React.createClass({ displayName: "\udc00" }),
  [`a\uD800b`]: React.createClass({ displayName: "a\ud800b" }),
  "𐀀": React.createClass({ displayName: "𐀀" }),
  ["\ud800"]: React.createClass({ displayName: "\ud800" })
};
