function test(foo) {
	var _foo$bar, _foo$bar2, _foo$bar3, _foo$bar4, _foo$bar5, _foo$bar6, _foo$bar6$baz, _foo$bar7, _foo$bar7$baz;
	foo === null || foo === void 0 ? void 0 : foo.bar;
	foo === null || foo === void 0 || (_foo$bar = foo.bar) === null || _foo$bar === void 0 ? void 0 : _foo$bar.baz;
	foo === null || foo === void 0 ? void 0 : foo(foo);
	foo === null || foo === void 0 ? void 0 : foo.bar();
	(_foo$bar2 = foo.bar) === null || _foo$bar2 === void 0 ? void 0 : _foo$bar2.call(foo, foo.bar, false);
	foo === null || foo === void 0 || (_foo$bar3 = foo.bar) === null || _foo$bar3 === void 0 ? void 0 : _foo$bar3.call(foo, foo.bar, true);
	(_foo$bar4 = foo.bar) === null || _foo$bar4 === void 0 ? void 0 : _foo$bar4.baz(foo.bar, false);
	foo === null || foo === void 0 || (_foo$bar5 = foo.bar) === null || _foo$bar5 === void 0 ? void 0 : _foo$bar5.baz(foo.bar, true);
	(_foo$bar6 = foo.bar) === null || _foo$bar6 === void 0 || (_foo$bar6$baz = _foo$bar6.baz) === null || _foo$bar6$baz === void 0 ? void 0 : _foo$bar6$baz.call(_foo$bar6, foo.bar, false);
	foo === null || foo === void 0 || (_foo$bar7 = foo.bar) === null || _foo$bar7 === void 0 || (_foo$bar7$baz = _foo$bar7.baz) === null || _foo$bar7$baz === void 0 ? void 0 : _foo$bar7$baz.call(_foo$bar7, foo.bar, true);
}
