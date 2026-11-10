try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    var x = "v";
    console.log(
      (function () {
        var x = "f";
        return x;
      })(),
      x,
    );
  }
  console.log(x);
}
