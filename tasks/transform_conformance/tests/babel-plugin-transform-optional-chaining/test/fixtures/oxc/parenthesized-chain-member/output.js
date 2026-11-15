var _user$profile$setting;
const user = undefined;
console.log((_user$profile$setting = (user === null || user === void 0 ? void 0 : user.profile).settings) === null || _user$profile$setting === void 0 ? void 0 : _user$profile$setting.theme);
function test(a, k) {
  var _a$b$k, _a$b, _a, _a$b$c, _a$b2, _a$b$c2, _a$b3, _a$b4;
  (_a$b$k = (a === null || a === void 0 ? void 0 : a.b)[k]) === null || _a$b$k === void 0 ? void 0 : _a$b$k.c;
  (_a$b = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a = a).b, _a))()) === null || _a$b === void 0 ? void 0 : _a$b.c;
  (_a$b$c = (_a$b2 = a === null || a === void 0 ? void 0 : a.b).c) === null || _a$b$c === void 0 ? void 0 : _a$b$c.call(_a$b2);
  (_a$b$c2 = (a === null || a === void 0 ? void 0 : a.b).c) === null || _a$b$c2 === void 0 ? true : delete _a$b$c2.d;
  a === null || a === void 0 || (_a$b3 = a.b) === null || _a$b3 === void 0 ? void 0 : _a$b3.c;
  a === null || a === void 0 || (_a$b4 = a.b) === null || _a$b4 === void 0 ? void 0 : _a$b4.call(a);
}
class Foo {
  #value;
  test(a) {
    var _a$b$value;
    return (_a$b$value = (a === null || a === void 0 ? void 0 : a.b).#value) === null || _a$b$value === void 0 ? void 0 : _a$b$value.c;
  }
}
