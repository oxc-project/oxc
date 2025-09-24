let x;
(function (_x) {
  console.log(x, y);
})(x || (x = {}));

x = /* @__PURE__ */ function (x) {
  x[x["y"] = 123] = "y";
  return x;
}(x || {});

var y = /* @__PURE__ */ function (y) {
  y[y["y"] = 123] = "y";
  return y;
}(y || {});

(function (_y) {
  console.log(x, y);
})(y || (y = {}));
