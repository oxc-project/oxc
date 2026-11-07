class Foo {
	static test() {
		var _getObj;
		_prop._ = babelHelpers.assertClassBrand(Foo, this, babelHelpers.assertClassBrand(Foo, this, _prop)._ + 1);
		_prop._ = babelHelpers.assertClassBrand(Foo, _getObj = getObj(), babelHelpers.assertClassBrand(Foo, _getObj, _prop)._ - 2);
	}
}
var _prop = { _: 1 };
