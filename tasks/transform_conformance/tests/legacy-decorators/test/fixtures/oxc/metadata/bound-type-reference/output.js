import { BoundTypeReference } from "./output";

var _ref;

console.log(BoundTypeReference);

let Example = class Example {
	constructor(count) {}
	prop = 1;
};

Example = babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, dce),
		babelHelpers.decorateMetadata("design:paramtypes", [
			typeof (_ref =
				typeof BoundTypeReference !== "undefined" && BoundTypeReference) ===
			"function"
				? _ref
				: Object,
		]),
	],
	Example,
);
