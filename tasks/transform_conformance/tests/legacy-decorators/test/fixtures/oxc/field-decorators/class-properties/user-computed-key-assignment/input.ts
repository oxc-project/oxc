// A variable assigned in a computed key is shared, so it isn't reused as the key's temp var.
let k;
const make = (n) =>
  class {
    [k = n] = 1;
  };
