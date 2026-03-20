var Runtime = function(Runtime) {
  Runtime[Runtime["X"] = Math.random()] = "X";
  Runtime[Runtime["Y"] = 1 + Runtime["X"]] = "Y";
  return Runtime;
}(Runtime || {});
Runtime.X;
