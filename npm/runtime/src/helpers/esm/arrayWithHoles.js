function _arrayWithHoles(r) {
  var t;
  if (Array.isArray(r) && ("undefined" == typeof Symbol || (t = r[Symbol.iterator]) === [][Symbol.iterator]) && (t || !(t = r["@@iterator"]) || t === []["@@iterator"])) return r;
}
export { _arrayWithHoles as default };