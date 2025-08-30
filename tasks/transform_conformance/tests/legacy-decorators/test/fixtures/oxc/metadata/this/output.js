let Example = class Example {
	constructor(a) {}
};
Example = babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, dce),
		babelHelpers.decorateMetadata("design:paramtypes", [Object]),
	],
	Example,
);
