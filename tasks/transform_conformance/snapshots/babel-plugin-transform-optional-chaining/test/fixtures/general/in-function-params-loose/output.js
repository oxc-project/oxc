function f(a = (() => {
	var _x;
	return (_x = x) === null || _x === void 0 ? void 0 : _x.y;
})()) {}
function g({ a, b = (() => {
	return a === null || a === void 0 ? void 0 : a.c;
})() }) {}
function h(a, { b = (() => {
	var _a$b;
	return (_a$b = a.b) === null || _a$b === void 0 || (_a$b = _a$b.c) === null || _a$b === void 0 ? void 0 : _a$b.d.e;
})() }) {}
function i(a, { b = (() => {
	var _a$b2;
	return (_a$b2 = a.b) === null || _a$b2 === void 0 || (_a$b2 = _a$b2.c) === null || _a$b2 === void 0 ? void 0 : _a$b2.d;
})().e }) {}
function j(a, { b = (() => {
	var _a$b3;
	return a === null || a === void 0 || (_a$b3 = a.b) === null || _a$b3 === void 0 ? void 0 : _a$b3.c().d.e;
})() }) {}
