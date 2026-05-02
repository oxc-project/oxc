class Foo {
	static test() {
		babelHelpers.assertClassBrand(Foo, this, _nullish)._ ?? (_nullish._ = babelHelpers.assertClassBrand(Foo, this, 1));
		babelHelpers.assertClassBrand(Foo, this, _and)._ && (_and._ = babelHelpers.assertClassBrand(Foo, this, 2));
		babelHelpers.assertClassBrand(Foo, this, _or)._ || (_or._ = babelHelpers.assertClassBrand(Foo, this, 3));
	}
}
var _nullish = { _: void 0 };
var _and = { _: 1 };
var _or = { _: void 0 };
