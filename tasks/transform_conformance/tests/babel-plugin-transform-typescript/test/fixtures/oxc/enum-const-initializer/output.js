const o = "o";
const alias = o + "k";
var E = /* @__PURE__ */ function(E) {
  E["A"] = "o";
  E[E["B"] = 1] = "B";
  E["C"] = "ok";
  return E;
}(E || {});
console.log(JSON.stringify(E));
