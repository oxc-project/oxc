function _setClassName(target, name) {
  var desc = Object.getOwnPropertyDescriptor(target, "name");
  // Static methods and accessors replace the name created by NamedEvaluation.
  if (!desc || "value" in desc && !desc.writable) {
    name = typeof name === "symbol" ? name.description === void 0 ? "" : "[" + name.description + "]" : String(name);
    Object.defineProperty(target, "name", { value: name, configurable: true });
  }
  return target;
}
export { _setClassName as default };
