// Bound root of member expression
let obj;
obj.prop **= 2;
obj['prop blah'] **= 3;
obj.foo.bar.qux **= 4;

let boundPropName;
obj[boundPropName] **= 5;
obj[unboundPropName] **= 6;

let boundPropName2;
obj.foo2.bar2[boundPropName2] **= 7;
obj.foo3.bar3[unboundPropName2] **= 8;

let boundPropObj;
obj[boundPropObj.foo.bar.qux] **= 9;
obj[unboundPropObj.foo.bar.qux] **= 10;

// Unbound root of member expression
unboundObj.prop **= 11;
unboundObj['prop blah'] **= 12;
unboundObj.foo.bar.qux **= 13;

let boundPropName3;
unboundObj[boundPropName3] **= 14;
unboundObj[unboundPropName3] **= 15;

let boundPropName4;
unboundObj.foo2.bar2[boundPropName4] **= 16;
unboundObj.foo3.bar3[unboundPropName4] **= 17;

let boundPropObj2;
unboundObj[boundPropObj2.foo.bar.qux] **= 18;
unboundObj[unboundPropObj2.foo.bar.qux] **= 19;

// Other expressions
let fn, fn2;
fn().prop **= 20;
fn().foo().bar().qux **= 21;
fn().prop[fn2()] **= 22;
fn().prop[fn3().foo().bar().qux() + ' junk'] **= 23;

// `this`
this.prop **= 24;
this.foo.bar.qux **= 25;
this['prop blah'] **= 26;
this[fn4().foo.bar.qux()] **= 27;

function outer() {
  this.prop **= 28;
  this.foo.bar.qux **= 29;
  this['prop blah'] **= 30;
  this[fn4().foo.bar.qux()] **= 31;
}

// Underscore var names
let ___bound;
___bound.prop **= 32;
___unbound.prop **= 33;
obj[___bound] **= 34;
obj[___unbound] **= 35;
