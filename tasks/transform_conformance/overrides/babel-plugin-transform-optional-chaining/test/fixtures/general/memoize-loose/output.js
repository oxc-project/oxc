function test(foo) {
  var _foo$bar, _foo$get, _foo$bar2, _foo$bar3, _foo$bar$baz, _foo$bar4, _foo$bar$baz2, _foo$bar5, _foo$bar6, _foo$bar7, _foo$bar8, _foo$bar9, _foo$bar10, _foo$bar10$baz, _foo$bar11, _foo$bar11$baz;
  foo === null || foo === void 0 ? void 0 : foo.bar;
  foo === null || foo === void 0 || (_foo$bar = foo.bar) === null || _foo$bar === void 0 ? void 0 : _foo$bar.baz;
  foo === null || foo === void 0 ? void 0 : foo(foo);
  foo === null || foo === void 0 ? void 0 : foo.bar();
  (_foo$get = foo.get(bar)) === null || _foo$get === void 0 ? void 0 : _foo$get();
  (_foo$bar2 = foo.bar()) === null || _foo$bar2 === void 0 ? void 0 : _foo$bar2();
  (_foo$bar3 = foo[bar]()) === null || _foo$bar3 === void 0 ? void 0 : _foo$bar3();
  (_foo$bar$baz = (_foo$bar4 = foo.bar()).baz) === null || _foo$bar$baz === void 0 ? void 0 : _foo$bar$baz.call(_foo$bar4);
  (_foo$bar$baz2 = (_foo$bar5 = foo[bar]()).baz) === null || _foo$bar$baz2 === void 0 ? void 0 : _foo$bar$baz2.call(_foo$bar5);
  (_foo$bar6 = foo.bar) === null || _foo$bar6 === void 0 ? void 0 : _foo$bar6.call(foo, foo.bar, false);
  foo === null || foo === void 0 || (_foo$bar7 = foo.bar) === null || _foo$bar7 === void 0 ? void 0 : _foo$bar7.call(foo, foo.bar, true);
  (_foo$bar8 = foo.bar) === null || _foo$bar8 === void 0 ? void 0 : _foo$bar8.baz(foo.bar, false);
  foo === null || foo === void 0 || (_foo$bar9 = foo.bar) === null || _foo$bar9 === void 0 ? void 0 : _foo$bar9.baz(foo.bar, true);
  (_foo$bar10 = foo.bar) === null || _foo$bar10 === void 0 || (_foo$bar10$baz = _foo$bar10.baz) === null || _foo$bar10$baz === void 0 ? void 0 : _foo$bar10$baz.call(_foo$bar10, foo.bar, false);
  foo === null || foo === void 0 || (_foo$bar11 = foo.bar) === null || _foo$bar11 === void 0 || (_foo$bar11$baz = _foo$bar11.baz) === null || _foo$bar11$baz === void 0 ? void 0 : _foo$bar11$baz.call(_foo$bar11, foo.bar, true);
}
