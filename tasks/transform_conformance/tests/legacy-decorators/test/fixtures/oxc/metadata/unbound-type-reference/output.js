var _ref;
let Example = class Example {
	constructor(count) {}
};

Example = babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, dce),
		babelHelpers.decorateMetadata("design:paramtypes", [
			typeof (_ref =
				typeof UnboundTypeReference !== "undefined" && UnboundTypeReference) ===
			"function"
				? _ref
				: Object,
		]),
	],
	Example,
);
