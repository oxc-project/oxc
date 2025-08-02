class Foo {
  // Regular uninitialized property
  a: string;
  
  // Property with definite assignment assertion
  b!: string;
  
  // Property with initializer (should be kept)
  c = "hello";
  
  // Optional property
  d?: string;
}