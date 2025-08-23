import { Foo } from "mod";
var _ref;

let Cls = class Cls {
	constructor(param, param2, param3, param4) {
		console.log(param, param2, param3, param4);
	}
};

Cls = babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, dec),
		babelHelpers.decorateMetadata("design:paramtypes", [
			typeof (_ref = typeof Foo !== "undefined" && Foo) === "function"
				? _ref
				: Object,
			Object,
			Object,
			Object,
		]),
	],
	Cls,
);
