var F = function(F) {
  F["A"] = foo + "a";
  F[F["B"] = typeof foo] = "B";
  F["C"] = `${foo}`;
  F[F["D"] = f()] = "D";
  F["E"] = foo + "a";
  F["G"] = "a" + foo + 1;
  F[F["H"] = 1 + foo] = "H";
  F["I"] = foo + "a";
  F[F["J"] = foo] = "J";
  F[F["K"] = "x"] = "K";
  F[F["L"] = "x"] = "L";
  F[F["M"] = "x"] = "M";
  F["N"] = "x";
  return F;
}(F || {});
