try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    try {
      throw "j";
    } catch (x) {
      var x = "v";
      console.log(x);
    }
    console.log(x);
  }
  console.log(x);
}
