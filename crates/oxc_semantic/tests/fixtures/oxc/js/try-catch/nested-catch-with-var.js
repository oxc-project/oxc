// https://github.com/oxc-project/oxc/issues/17307
// var declaration in nested catch block should create hoisted binding
try {} catch (e) {
  try {} catch (e) {
    var e = "e";
    console.log(e === "e");
  }
}
console.log(e === undefined);
