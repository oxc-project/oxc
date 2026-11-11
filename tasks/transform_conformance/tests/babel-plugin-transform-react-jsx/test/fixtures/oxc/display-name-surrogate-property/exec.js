const React = { createClass: (value) => value };
const obj = {};
obj["\uD800"] = React.createClass({});
obj["\uDC00"] = React.createClass({});
obj[`a\uD800b`] = React.createClass({});
obj["\uD800\uDC00"] = React.createClass({});
for (const name of Object.keys(obj)) {
  expect(obj[name].displayName).toBe(name);
}
