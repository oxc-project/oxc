var _objectSpread = require("@babel/runtime/helpers/objectSpread2");
var k = {
	a: 1,
	b: 2
};
var o = _objectSpread(_objectSpread({ a: 3 }, k), {}, { b: k.a++ });
var pureA = {};
var pureB = {};
var pureC = {};
var pureD = {};
var pureE = {};
function impureFunc() {
	console.log("hello");
}
var output = _objectSpread(_objectSpread(_objectSpread(_objectSpread(_objectSpread(_objectSpread({}, pureA), {}, {
	get foo() {},
	get bar() {}
}, pureB), pureC), impureFunc()), pureD), {}, { pureD });
var simpleOutput = _objectSpread(_objectSpread({}, pureA), {}, { test: "1" }, pureB);
