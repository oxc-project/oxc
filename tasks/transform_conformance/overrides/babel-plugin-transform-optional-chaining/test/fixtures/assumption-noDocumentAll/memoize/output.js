function test(foo) {
	var _foo$bar, _foo$get, _foo$bar2, _foo$bar3, _foo$bar$baz, _foo$bar$baz2, _foo$bar4, _foo$bar5, _foo$bar6, _foo$bar7, _foo$bar8, _foo$bar8$baz, _foo$bar9, _foo$bar9$baz;
	foo == null ? void 0 : foo.bar;
	foo == null || (_foo$bar = foo.bar) == null ? void 0 : _foo$bar.baz;
	foo == null ? void 0 : foo(foo);
	foo == null ? void 0 : foo.bar();
	(_foo$get = foo.get(bar)) == null ? void 0 : _foo$get();
	(_foo$bar2 = foo.bar()) == null ? void 0 : _foo$bar2();
	(_foo$bar3 = foo[bar]()) == null ? void 0 : _foo$bar3();
	(_foo$bar$baz = foo.bar().baz) == null ? void 0 : _foo$bar$baz();
	(_foo$bar$baz2 = foo[bar]().baz) == null ? void 0 : _foo$bar$baz2();
	(_foo$bar4 = foo.bar) == null ? void 0 : _foo$bar4.call(foo, foo.bar, false);
	foo == null || (_foo$bar5 = foo.bar) == null ? void 0 : _foo$bar5.call(foo, foo.bar, true);
	(_foo$bar6 = foo.bar) == null ? void 0 : _foo$bar6.baz(foo.bar, false);
	foo == null || (_foo$bar7 = foo.bar) == null ? void 0 : _foo$bar7.baz(foo.bar, true);
	(_foo$bar8 = foo.bar) == null || (_foo$bar8$baz = _foo$bar8.baz) == null ? void 0 : _foo$bar8$baz.call(_foo$bar8, foo.bar, false);
	foo == null || (_foo$bar9 = foo.bar) == null || (_foo$bar9$baz = _foo$bar9.baz) == null ? void 0 : _foo$bar9$baz.call(_foo$bar9, foo.bar, true);
}
