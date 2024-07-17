let a;
// the default param value is resolved to the outer scope
let foo = function (b = a) {
  let a;
};
