import { Foo } from "mod";

let Cls = class Cls {
	constructor(param, param2, param3, param4) {
		console.log(param, param2, param3, param4);
	}
};

Cls = babelHelpers.decorate(
	[
		babelHelpers.decorateParam(0, dec),
		babelHelpers.decorateMetadata("design:paramtypes", [
			Foo,
			Object,
			Object,
			Object,
		]),
	],
	Cls,
);
