var A = /*#__PURE__*/function (A) {
  A[A["a"] = Infinity] = "a";
  A[A["b"] = Infinity] = "b";
  A[A["c"] = Infinity] = "c";
  A["d"] = "Infinitytest";
  A[A["e"] = -Infinity] = "e";
  return A;
}(A || {});
var B = /*#__PURE__*/function (B) {
  B[B["a"] = NaN] = "a";
  B[B["b"] = NaN] = "b";
  B[B["c"] = NaN] = "c";
  B["d"] = "nanNaN";
  B[B["e"] = NaN] = "e";
  return B;
}(B || {});
var C = /*#__PURE__*/function (C) {
  C["a"] = "test100000000000000000000";
  C["b"] = "1e+30test";
  C["c"] = "test1234567890987test";
  return C;
}(C || {});
var D = /*#__PURE__*/function (D) {
  D["a"] = "hello";
  D[D["b"] = NaN] = "b";
  D[D["c"] = -1] = "c";
  return D;
}(D || {});
