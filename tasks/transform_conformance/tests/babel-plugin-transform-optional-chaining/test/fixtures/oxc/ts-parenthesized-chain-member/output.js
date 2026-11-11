function test(a, k) {
  var _c, _k, _ref, _a, _c2, _ref2, _ref3, _ref4, _a2;
  (_c = (a === null || a === void 0 ? void 0 : a.b).c) === null || _c === void 0 ? void 0 : _c.d;
  (_k = (a === null || a === void 0 ? void 0 : a.b)[k]) === null || _k === void 0 ? void 0 : _k.c;
  (_ref = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a = a).b, _a))()) === null || _ref === void 0 ? void 0 : _ref.c;
  (_c2 = (_ref2 = a === null || a === void 0 ? void 0 : a.b).c) === null || _c2 === void 0 ? void 0 : _c2.call(_ref2);
  a === null || a === void 0 || (_ref3 = a.b) === null || _ref3 === void 0 ? void 0 : _ref3.c;
  (_ref4 = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a2 = a).b, _a2))(k())) === null || _ref4 === void 0 ? void 0 : _ref4.c;
  (a === null || a === void 0 ? void 0 : a())();
}
