var _objectDestructuringEmpty = require("@babel/runtime/helpers/objectDestructuringEmpty");
var _extends = require("@babel/runtime/helpers/extends");
it("es7.objectRestSpread", () => {
	let original = {
		a: 1,
		b: 2
	};
	let copy = _extends({}, (_objectDestructuringEmpty(original), original));
});
