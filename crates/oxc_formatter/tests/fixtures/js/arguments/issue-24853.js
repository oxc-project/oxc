// issue #24853: a call-like second argument is "hopefully short" only when it
// has at most one argument and its whole callee chain is simple.
const schema = z.preprocess(val => {
  if (typeof val !== "string") return val;
  return val.trim();
}, z.enum(["first_value", "second_value", "third_value", "fourth_value"]).nullable().optional());

foo(val => {
  if (typeof val !== "string") return val;
  return val.trim();
}, new Foo(firstArgument, secondArgument));

// A simple zero-argument call still allows hugging the first argument.
run(val => {
  return val;
}, cleanup());
