try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    var x = "v";
    console.log(x);
    var x = "w";
    console.log(x);
  }
  console.log(x);
}
