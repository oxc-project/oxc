var _unbound, _unboundObj, _boundObj$foo$bar, _unboundObj$foo$bar, _boundProp, _unboundProp, _unboundObj2, _boundProp2, _unboundObj3, _unboundProp2;
let bound, boundObj, boundProp;

x = bound = Math.pow(bound, 1);
x = (_unbound = unbound, unbound = Math.pow(_unbound, 2));

x = boundObj["prop"] = Math.pow(boundObj["prop"], 3);
x = (_unboundObj = unboundObj, _unboundObj["prop"] = Math.pow(_unboundObj["prop"], 4));
x = (_boundObj$foo$bar = boundObj.foo.bar, _boundObj$foo$bar["qux"] = Math.pow(_boundObj$foo$bar["qux"], 5));
x = (_unboundObj$foo$bar = unboundObj.foo.bar, _unboundObj$foo$bar["qux"] = Math.pow(_unboundObj$foo$bar["qux"], 6));

x = (_boundProp = boundProp, boundObj[_boundProp] = Math.pow(boundObj[_boundProp], 7));
x = (_unboundProp = unboundProp, boundObj[_unboundProp] = Math.pow(boundObj[_unboundProp], 8));
x = (_unboundObj2 = unboundObj, _boundProp2 = boundProp, _unboundObj2[_boundProp2] = Math.pow(_unboundObj2[_boundProp2], 9));
x = (_unboundObj3 = unboundObj, _unboundProp2 = unboundProp, _unboundObj3[_unboundProp2] = Math.pow(_unboundObj3[_unboundProp2], 10));
