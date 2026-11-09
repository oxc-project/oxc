try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    console.log(x);
    var x = "v";
    console.log(x);
  }
  console.log(x);
}
console.log(x);
