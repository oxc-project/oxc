// Panicked when transforming optional chaining with TS non-null assertion inside a computed key.
// https://github.com/oxc-project/oxc/issues/22246
var _a$b$c, _b, _a, _b2, _a$d, _b3;
const x = (_a$b$c = a[(_b = b) === null || _b === void 0 ? void 0 : _b.c]) === null || _a$b$c === void 0 ? void 0 : _a$b$c["d"];
const y = (_a = a) === null || _a === void 0 || (_a = _a[(_b2 = b) === null || _b2 === void 0 ? void 0 : _b2.c]) === null || _a === void 0 ? void 0 : _a.d;
const z = (_a$d = a[(_b3 = b) === null || _b3 === void 0 ? void 0 : _b3.c.d]) === null || _a$d === void 0 ? void 0 : _a$d.e;
