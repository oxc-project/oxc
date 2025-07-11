class DeclareFields {}

babelHelpers.decorate(
	[dec, babelHelpers.decorateMetadata("design:type", Number)],
	DeclareFields.prototype,
	"name",
	void 0,
);

babelHelpers.decorate(
	[dec, babelHelpers.decorateMetadata("design:type", String)],
	DeclareFields.prototype,
	"output",
	void 0,
);
