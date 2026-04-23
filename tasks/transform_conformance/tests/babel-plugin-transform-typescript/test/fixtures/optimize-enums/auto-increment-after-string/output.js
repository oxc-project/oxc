var Mixed = /* @__PURE__ */ function(Mixed) {
  Mixed["A"] = "hello";
  Mixed[Mixed["B"] = 1 + Mixed["A"]] = "B";
  return Mixed;
}(Mixed || {});
"hello";
Mixed.B;
