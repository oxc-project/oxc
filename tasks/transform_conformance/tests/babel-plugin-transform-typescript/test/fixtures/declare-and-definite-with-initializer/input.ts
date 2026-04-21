// Test declare fields with initializers vs definite assignment assertions
class DeclareExample {
  declare readonly bar = "test";
  declare readonly foo = 1;
}

class DefiniteExample {
   readonly bar! = "test";
   readonly foo! = 1;
}