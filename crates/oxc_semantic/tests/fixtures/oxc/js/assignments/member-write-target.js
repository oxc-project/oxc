// MemberWriteTarget: compound assignment
let a = {};
a.x += 1;
a.x -= 1;
a.x &&= 1;

// MemberWriteTarget: delete
let b = {};
delete b.x;

// No MemberWriteTarget: conditional test position
let c = {};
(c ? 0 : 1).foo = 1;

// MemberWriteTarget: conditional consequent/alternate
let d = {};
let e = {};
(true ? d : e).foo = 1;

// No MemberWriteTarget: computed property key
let key;
this[key] = 1;
this[key] += 1;
delete this[key];

// MemberWriteTarget: computed object
let f = {};
f[key] = 1;
f[key] += 1;
