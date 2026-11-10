var x = "top";
try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    var x = "v";
    console.log(x);
  }
  console.log(x);
}
console.log(x);
