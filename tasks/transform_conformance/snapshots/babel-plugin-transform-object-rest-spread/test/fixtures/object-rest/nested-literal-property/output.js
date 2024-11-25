var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
let useState = [{ some: 42 }, () => null];
let { 0: _ref, "2": _ref2, 1: setState } = useState, { numeric } = _ref, rest1 = _objectWithoutProperties(_ref, ["numeric"]), { str } = _ref2, rest2 = _objectWithoutProperties(_ref2, ["str"]);
