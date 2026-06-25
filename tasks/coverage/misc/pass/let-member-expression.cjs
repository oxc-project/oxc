// Test that `let` can be used as identifier in member expression or call expression in script
let.x;

let.x = 1;
let()[x] = 1;

let?.x;
let?.y.z;
let?.[0];
let?.method();
