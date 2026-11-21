// A variable assigned in a computed key is shared, so it isn't reused as the key's temp var.
let k;
const make = (n) => {
  let _k;
  return _k = k = n, class {
    constructor() {
      babelHelpers.defineProperty(this, _k, 1);
    }
  };
};

