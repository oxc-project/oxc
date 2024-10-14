let bound, boundObj, boundProp;

x = bound **= 1;
x = unbound **= 2;

x = boundObj.prop **= 3;
x = unboundObj.prop **= 4;
x = boundObj.foo.bar.qux **= 5;
x = unboundObj.foo.bar.qux **= 6;

x = boundObj[boundProp] **= 7;
x = boundObj[unboundProp] **= 8;
x = unboundObj[boundProp] **= 9;
x = unboundObj[unboundProp] **= 10;
