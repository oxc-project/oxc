let Foo = class Foo {
	method1(param) {
		return !!param;
	}
	method2(param) {
		return !!param;
	}
  constructor(param) {}
	method3(param) {
		return !!param;
	}
	method4(param, param2) {
		return !!param;
	}
}

babelHelpers.decorate(
	[
		methodDecorator(1),
		methodDecorator(2),
		babelHelpers.decorateParam(0, paramDecorator),
		babelHelpers.decorateMetadata("design:type", Function),
		babelHelpers.decorateMetadata("design:paramtypes", [String]),
		babelHelpers.decorateMetadata("design:returntype", Boolean),
	],
	Foo.prototype,
	"method1",
	null,
);

babelHelpers.decorate(
	[
		methodDecorator(1),
		methodDecorator(2),
		babelHelpers.decorateMetadata("design:type", Function),
		babelHelpers.decorateMetadata("design:paramtypes", [String]),
		babelHelpers.decorateMetadata("design:returntype", Boolean),
	],
	Foo.prototype,
	"method2",
	null,
);

babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, paramDecorator),
		babelHelpers.decorateMetadata("design:type", Function),
		babelHelpers.decorateMetadata("design:paramtypes", [String]),
		babelHelpers.decorateMetadata("design:returntype", Boolean),
	],
	Foo.prototype,
	"method3",
	null,
);

babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, paramDecorator),
		babelHelpers.decorateParam(1, paramDecorator),
		babelHelpers.decorateMetadata("design:type", Function),
		babelHelpers.decorateMetadata("design:paramtypes", [String, String]),
		babelHelpers.decorateMetadata("design:returntype", Boolean),
	],
	Foo.prototype,
	"method4",
	null,
);
Foo = babelHelpers.decorate([babelHelpers.decorateParam(0, paramDecorator), babelHelpers.decorateMetadata("design:paramtypes", [Number])], Foo);
export { Foo };