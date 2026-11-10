try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    var x = "v";
    console.log(x);
  }
}
