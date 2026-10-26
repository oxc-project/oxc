function* generatorInterface() {
  interface I { [yield: string]: unknown }
}
function* generatorClass() {
  class C { [yield: string]: unknown }
}
async function asyncClass() {
  class C { [await: string]: unknown }
}
