var _ref;

let Cls = class Cls {
	constructor(param, param2) {
		console.log(param, param2);
	}
};

Cls = babelHelpers.decorate(
	[
		babelHelpers.decorateMetadata("design:paramtypes", [
			typeof (_ref = typeof Foo !== "undefined" && Foo) === "function"
				? _ref
				: Object,
			Object,
		]),
		babelHelpers.decorateParam(0, dec),
	],
	Cls,
);

export {};
