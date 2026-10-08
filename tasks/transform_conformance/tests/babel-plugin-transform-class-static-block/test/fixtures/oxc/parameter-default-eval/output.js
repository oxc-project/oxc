var _staticBlock;
function preserveEval({ value = (eval("var exposed = 7"), class {
  static #_ = _staticBlock = () => ((() => {})(), this);
}, _staticBlock()) } = {}, other = exposed) {
  return other;
}
const result = preserveEval({});
