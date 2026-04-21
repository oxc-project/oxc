// Test that `let` cannot be used as identifier in member expression or call expression in module
'use strict';

let.x;

let.x = 1;
let()[x] = 1;

let?.x;
let?.y.z;
let?.[0];
let?.method();
