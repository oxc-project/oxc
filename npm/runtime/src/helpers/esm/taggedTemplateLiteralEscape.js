function _taggedTemplateLiteralEscape(e, t) {
  return t || (t = e.slice(0)), Object.freeze(Object.defineProperty(e, "raw", {
    value: Object.freeze(t)
  }));
}
export { _taggedTemplateLiteralEscape as default };
