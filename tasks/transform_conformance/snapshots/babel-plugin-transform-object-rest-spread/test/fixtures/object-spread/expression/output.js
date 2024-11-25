var _objectSpread = require("@babel/runtime/helpers/objectSpread2");
var a;
var b;
var c;
var d;
var x;
var y;
_objectSpread(_objectSpread(_objectSpread({ x }, y), {}, { a }, b), {}, { c });
_objectSpread({}, Object.prototype);
_objectSpread({}, { foo: "bar" });
_objectSpread(_objectSpread({}, { foo: "bar" }), { bar: "baz" });
_objectSpread({}, { get foo() {
	return "foo";
} });
