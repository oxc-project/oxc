let a;
// the default param value is resolved to the outer scope
let foo = (b = a) => {
  let a;
};
