var _foo, _foo2, _foo$bar, _foo3, _foo4, _foo4$bar;
(_foo = foo) === null || _foo === void 0 ? void 0 : _foo(foo);
(_foo2 = foo) === null || _foo2 === void 0 ? void 0 : _foo2.bar();
(_foo$bar = (_foo3 = foo).bar) === null || _foo$bar === void 0 ? void 0 : _foo$bar.call(_foo3, foo.bar, false);
(_foo4 = foo) === null || _foo4 === void 0 || (_foo4$bar = _foo4.bar) === null || _foo4$bar === void 0 ? void 0 : _foo4$bar.call(_foo4, foo.bar, true);
