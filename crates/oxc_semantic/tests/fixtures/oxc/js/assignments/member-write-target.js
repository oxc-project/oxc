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
