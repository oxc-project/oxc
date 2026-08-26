const _excluded = ["value"];
const rest = "outer";

function readAfter(_ref) {
  let { value } = _ref,
    rest = babelHelpers.objectWithoutProperties(_ref, _excluded);
  let selected =
    arguments.length > 1 && arguments[1] !== undefined
      ? arguments[1]
      : value;
  return [rest, selected];
}

function readBefore() {
  let selected =
    arguments.length > 0 && arguments[0] !== undefined
      ? arguments[0]
      : rest;
  let _ref2 = arguments.length > 1 ? arguments[1] : undefined;
  let rest = babelHelpers.extends(
    {},
    (babelHelpers.objectDestructuringEmpty(_ref2), _ref2),
  );
  return selected;
}
