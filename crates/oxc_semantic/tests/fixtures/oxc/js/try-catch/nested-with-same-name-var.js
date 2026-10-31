try {
  throw 1
} catch (e) {
  try {
    throw 2
  } catch (e) {
    var e = 'inner';
    console.log(e);
  }
}
console.log(e);
