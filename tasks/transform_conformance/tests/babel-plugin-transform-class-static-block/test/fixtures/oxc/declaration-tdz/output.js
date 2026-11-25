var _staticBlock;
const read = () => A;
let A = (class A {
  static #_ = _staticBlock = () => ((() => {
    try {
      read();
      console.log("initialized");
    } catch (e) {
      console.log(e.name);
    }
  })(), this);
}, _staticBlock());
