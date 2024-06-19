function foo() {
   return 1;
}
// inferred type is number

function bar() {
  if (true) {
   return;
  }
  return 1;
}
// inferred type is number | undefined

function baz() {
 if (true) {
  return null;
 }
 return 1;
}
// We can't infer return type if there are multiple return statements with different types