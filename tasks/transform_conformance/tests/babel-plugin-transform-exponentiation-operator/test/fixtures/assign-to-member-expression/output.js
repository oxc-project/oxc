var _obj$foo$bar, _boundPropName, _unboundPropName, _obj$foo2$bar, _boundPropName2, _obj$foo3$bar, _unboundPropName2, _boundPropObj$foo$bar, _unboundPropObj$foo$b, _unboundObj, _unboundObj2, _unboundObj$foo$bar, _unboundObj3, _boundPropName3, _unboundObj4, _unboundPropName3, _unboundObj$foo2$bar, _boundPropName4, _unboundObj$foo3$bar, _unboundPropName4, _unboundObj5, _boundPropObj2$foo$ba, _unboundObj6, _unboundPropObj2$foo$, _fn, _fn$foo$bar, _fn$prop, _fn2, _fn$prop2, _ref, _this, _this$foo$bar, _this2, _this3, _fn4$foo$bar$qux, _unbound, _bound, _unbound2;

// Bound root of member expression
let obj;
obj["prop"] = Math.pow(obj["prop"], 2);
obj["prop blah"] = Math.pow(obj["prop blah"], 3);
(_obj$foo$bar = obj.foo.bar),
  (_obj$foo$bar["qux"] = Math.pow(_obj$foo$bar["qux"], 4));
let boundPropName;
(_boundPropName = boundPropName),
  (obj[_boundPropName] = Math.pow(obj[_boundPropName], 5));
(_unboundPropName = unboundPropName),
  (obj[_unboundPropName] = Math.pow(obj[_unboundPropName], 6));
let boundPropName2;
(_obj$foo2$bar = obj.foo2.bar2),
  (_boundPropName2 = boundPropName2),
  (_obj$foo2$bar[_boundPropName2] = Math.pow(
    _obj$foo2$bar[_boundPropName2],
    7
  ));
(_obj$foo3$bar = obj.foo3.bar3),
  (_unboundPropName2 = unboundPropName2),
  (_obj$foo3$bar[_unboundPropName2] = Math.pow(
    _obj$foo3$bar[_unboundPropName2],
    8
  ));
let boundPropObj;
(_boundPropObj$foo$bar = boundPropObj.foo.bar.qux),
  (obj[_boundPropObj$foo$bar] = Math.pow(obj[_boundPropObj$foo$bar], 9));
(_unboundPropObj$foo$b = unboundPropObj.foo.bar.qux),
  (obj[_unboundPropObj$foo$b] = Math.pow(obj[_unboundPropObj$foo$b], 10));

// Unbound root of member expression
(_unboundObj = unboundObj),
  (_unboundObj["prop"] = Math.pow(_unboundObj["prop"], 11));
(_unboundObj2 = unboundObj),
  (_unboundObj2["prop blah"] = Math.pow(_unboundObj2["prop blah"], 12));
(_unboundObj$foo$bar = unboundObj.foo.bar),
  (_unboundObj$foo$bar["qux"] = Math.pow(_unboundObj$foo$bar["qux"], 13));
let boundPropName3;
(_unboundObj3 = unboundObj),
  (_boundPropName3 = boundPropName3),
  (_unboundObj3[_boundPropName3] = Math.pow(_unboundObj3[_boundPropName3], 14));
(_unboundObj4 = unboundObj),
  (_unboundPropName3 = unboundPropName3),
  (_unboundObj4[_unboundPropName3] = Math.pow(
    _unboundObj4[_unboundPropName3],
    15
  ));
let boundPropName4;
(_unboundObj$foo2$bar = unboundObj.foo2.bar2),
  (_boundPropName4 = boundPropName4),
  (_unboundObj$foo2$bar[_boundPropName4] = Math.pow(
    _unboundObj$foo2$bar[_boundPropName4],
    16
  ));
(_unboundObj$foo3$bar = unboundObj.foo3.bar3),
  (_unboundPropName4 = unboundPropName4),
  (_unboundObj$foo3$bar[_unboundPropName4] = Math.pow(
    _unboundObj$foo3$bar[_unboundPropName4],
    17
  ));
let boundPropObj2;
(_unboundObj5 = unboundObj),
  (_boundPropObj2$foo$ba = boundPropObj2.foo.bar.qux),
  (_unboundObj5[_boundPropObj2$foo$ba] = Math.pow(
    _unboundObj5[_boundPropObj2$foo$ba],
    18
  ));
(_unboundObj6 = unboundObj),
  (_unboundPropObj2$foo$ = unboundPropObj2.foo.bar.qux),
  (_unboundObj6[_unboundPropObj2$foo$] = Math.pow(
    _unboundObj6[_unboundPropObj2$foo$],
    19
  ));

// Other expressions
let fn, fn2;
(_fn = fn()), (_fn["prop"] = Math.pow(_fn["prop"], 20));
(_fn$foo$bar = fn().foo().bar()),
  (_fn$foo$bar["qux"] = Math.pow(_fn$foo$bar["qux"], 21));
(_fn$prop = fn().prop),
  (_fn2 = fn2()),
  (_fn$prop[_fn2] = Math.pow(_fn$prop[_fn2], 22));
(_fn$prop2 = fn().prop),
  (_ref = fn3().foo().bar().qux() + " junk"),
  (_fn$prop2[_ref] = Math.pow(_fn$prop2[_ref], 23));

// `this`
(_this = this), (_this["prop"] = Math.pow(_this["prop"], 24));
(_this$foo$bar = this.foo.bar),
  (_this$foo$bar["qux"] = Math.pow(_this$foo$bar["qux"], 25));
(_this2 = this), (_this2["prop blah"] = Math.pow(_this2["prop blah"], 26));
(_this3 = this),
  (_fn4$foo$bar$qux = fn4().foo.bar.qux()),
  (_this3[_fn4$foo$bar$qux] = Math.pow(_this3[_fn4$foo$bar$qux], 27));

function outer() {
  var _this4, _this$foo$bar2, _this5, _this6, _fn4$foo$bar$qux2;
  (_this4 = this), (_this4["prop"] = Math.pow(_this4["prop"], 28));
  (_this$foo$bar2 = this.foo.bar),
  (_this$foo$bar2["qux"] = Math.pow(_this$foo$bar2["qux"], 29));
  (_this5 = this), (_this5["prop blah"] = Math.pow(_this5["prop blah"], 30));
  (_this6 = this),
  (_fn4$foo$bar$qux2 = fn4().foo.bar.qux()),
  (_this6[_fn4$foo$bar$qux2] = Math.pow(_this6[_fn4$foo$bar$qux2], 31));
}

// Underscore var names
let ___bound;
___bound["prop"] = Math.pow(___bound["prop"], 32);
(_unbound = ___unbound), (_unbound["prop"] = Math.pow(_unbound["prop"], 33));
(_bound = ___bound), (obj[_bound] = Math.pow(obj[_bound], 34));
(_unbound2 = ___unbound), (obj[_unbound2] = Math.pow(obj[_unbound2], 35));
