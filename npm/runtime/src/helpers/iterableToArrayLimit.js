function _iterableToArrayLimit(r, t) {
  var e = null == r ? null : "undefined" != typeof Symbol && r[Symbol.iterator] || r["@@iterator"];
  if (Array.isArray(r) && (null == e || e === ("undefined" != typeof Symbol && [][Symbol.iterator]) || e === []["@@iterator"])) return r;
  if (null != e) {
    var l,
      n,
      i,
      a,
      u = [],
      o = !0,
      f = !1;
    try {
      if (i = (e = e.call(r)).next, 0 === t) {
        if (Object(e) !== e) return;
        o = !1;
      } else for (; !(o = (l = i.call(e)).done) && (u.push(l.value), u.length !== t); o = !0);
    } catch (r) {
      f = !0, n = r;
    } finally {
      try {
        if (!o && null != e["return"] && (a = e["return"](), Object(a) !== a)) return;
      } finally {
        if (f) throw n;
      }
    }
    return u;
  }
}
module.exports = _iterableToArrayLimit, module.exports.__esModule = true, module.exports["default"] = module.exports;