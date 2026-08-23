import _typeof from "./typeof.js";
import setPrototypeOf from "./setPrototypeOf.js";
import inherits from "./inherits.js";
function _wrapRegExp() {
  _wrapRegExp = function _wrapRegExp(e, r) {
    return new BabelRegExp(e, void 0, r);
  };
  var e = RegExp.prototype,
    r = new WeakMap(),
    n = Object.prototype.hasOwnProperty;
  function BabelRegExp(e, t, p) {
    var o = RegExp(e, t);
    return r.set(o, p || r.get(e)), setPrototypeOf(o, BabelRegExp.prototype);
  }
  function buildGroups(e, t) {
    var p = r.get(t);
    return Object.keys(p).reduce(function (r, t) {
      var o = p[t];
      if ("number" == typeof o) r[t] = e[o];else {
        for (var i = 0; void 0 === e[o[i]] && i + 1 < o.length;) i++;
        r[t] = e[o[i]];
      }
      return r;
    }, Object.create(null));
  }
  function buildGroupReplacement(e) {
    return "number" == typeof e ? "$" + (e < 10 ? "0" : "") + e : e.map(buildGroupReplacement).join("");
  }
  function getGroupReplacement(e, r) {
    return n.call(e, r) ? buildGroupReplacement(e[r]) : "";
  }
  function cloneForMatchAll(e) {
    var t = new BabelRegExp(e, e.flags);
    return t.lastIndex = e.lastIndex, t.constructor = BabelRegExp, t;
  }
  return inherits(BabelRegExp, RegExp), BabelRegExp.prototype.constructor = RegExp, BabelRegExp.prototype.exec = function (t) {
    var p = e.exec.call(this, t);
    if (p && r.get(this)) {
      p.groups = buildGroups(p, this);
      var o = p.indices;
      o && (o.groups = buildGroups(o, this));
    }
    return p;
  }, Symbol.matchAll && (BabelRegExp.prototype[Symbol.matchAll] = function (t) {
    return e[Symbol.matchAll].call(cloneForMatchAll(this), t);
  }), BabelRegExp.prototype[Symbol.replace] = function (t, p) {
    var o = r.get(this);
    if (!o) return e[Symbol.replace].call(this, t, p);
    if ("string" == typeof p) {
      return e[Symbol.replace].call(this, t, p.replace(/\$\$|\$<([^>]+)(>|$)/g, function (e, r, t) {
        if ("$$" === e) return e;
        if ("" === t) return e;
        return getGroupReplacement(o, r);
      }));
    }
    if ("function" == typeof p) {
      var i = this;
      return e[Symbol.replace].call(this, t, function () {
        var e = arguments;
        return "object" != _typeof(e[e.length - 1]) && (e = [].slice.call(e)).push(buildGroups(e, i)), p.apply(this, e);
      });
    }
    return e[Symbol.replace].call(this, t, p);
  }, _wrapRegExp.apply(this, arguments);
}
export { _wrapRegExp as default };
