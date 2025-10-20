// Examples of incorrect code for no-confusing-void-expression rule

// arrow function returning void expression
const foo = () => void bar();

// conditional expression
const result = condition ? void foo() : bar();

// void in conditional
if (void foo()) {
  // ...
}
